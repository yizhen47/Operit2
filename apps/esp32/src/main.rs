#![allow(non_snake_case)]

mod app;
mod config;
mod status;
mod ui;

#[cfg(target_os = "espidf")]
mod web;
#[cfg(target_os = "espidf")]
mod wifi;

/// Starts the ESP32-2432S028 Operit Edge firmware.
#[cfg(target_os = "espidf")]
fn main() {
    if let Err(error) = runFirmware() {
        log::error!("operit-esp32 failed: {}", error.message);
        panic!("operit-esp32 failed: {}", error.message);
    }
}

/// Stops non-ESP-IDF targets from launching the firmware binary.
#[cfg(not(target_os = "espidf"))]
fn main() {
    eprintln!("operit-esp32 requires the xtensa-esp32-espidf target");
    std::process::exit(1);
}

#[cfg(target_os = "espidf")]
fn runFirmware() -> operit_host_api::HostResult<()> {
    use std::future::Future;
    use std::sync::Arc;
    use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

    use esp_idf_hal::delay::FreeRtos;
    use esp_idf_hal::peripherals::Peripherals;
    use operit_board_esp32::{Esp32Board, INITIAL_EXPRESSION, LED_GREEN_PIN, LED_RED_PIN};
    use operit_host_api::HostError;
    use operit_node_edge::{createDeviceIoService, createRobotFaceService, EdgeNode};
    use operit_proxy_edge::{EdgeProxy, EdgeProxyError};

    fn edgeError(error: EdgeProxyError) -> HostError {
        HostError::new(error.to_string())
    }

    fn noopWaker() -> Waker {
        unsafe fn clone(_: *const ()) -> RawWaker {
            noopRawWaker()
        }
        unsafe fn wake(_: *const ()) {}
        unsafe fn wakeByRef(_: *const ()) {}
        unsafe fn drop(_: *const ()) {}

        fn noopRawWaker() -> RawWaker {
            static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wakeByRef, drop);
            RawWaker::new(std::ptr::null(), &VTABLE)
        }

        unsafe { Waker::from_raw(noopRawWaker()) }
    }

    fn blockOn<F: Future>(future: F) -> F::Output {
        let waker = noopWaker();
        let mut context = Context::from_waker(&waker);
        let mut future = std::pin::pin!(future);
        loop {
            match future.as_mut().poll(&mut context) {
                Poll::Ready(output) => return output,
                Poll::Pending => FreeRtos::delay_ms(1),
            }
        }
    }

    use crate::app::Esp32App;
    use crate::config::Esp32FirmwareConfig;
    use crate::status::FirmwareStatus;
    use crate::web::Esp32WebHome;
    use crate::wifi::Esp32Wifi;

    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let config = Esp32FirmwareConfig::fromEnv();
    let peripherals = Peripherals::take().map_err(|error| HostError::new(error.to_string()))?;
    let modem = peripherals.modem;
    let mut board = Esp32Board::new(
        peripherals.spi2,
        peripherals.pins.gpio2,
        peripherals.pins.gpio12,
        peripherals.pins.gpio13,
        peripherals.pins.gpio14,
        peripherals.pins.gpio15,
        peripherals.pins.gpio21,
        peripherals.pins.gpio4,
        peripherals.pins.gpio16,
        peripherals.pins.gpio17,
        peripherals.spi3,
        peripherals.pins.gpio25,
        peripherals.pins.gpio32,
        peripherals.pins.gpio39,
        peripherals.pins.gpio33,
        peripherals.pins.gpio36,
    )?;
    let hostManager = board.installIntoHostManager();
    let screenMirror = board.screenMirror();
    let edgeNode = EdgeNode::new(createDeviceIoService(hostManager.clone()))
        .withRobotFaceService(createRobotFaceService(hostManager));
    let edgeProxy = EdgeProxy::new(edgeNode);
    let status = Arc::new(FirmwareStatus::new(INITIAL_EXPRESSION));
    let mut app = Esp32App::new(edgeProxy, Arc::clone(&status));
    blockOn(app.setExpression("booting")).map_err(edgeError)?;
    let _wifi = if config.hasWifi() {
        match Esp32Wifi::connect(modem, &config) {
            Ok(wifi) => {
                let ip = wifi.ipv4()?;
                status.setWifiSsid(config.wifiSsid.clone());
                status.setIpv4(ip.to_string());
                blockOn(app.setExpression("online")).map_err(edgeError)?;
                blockOn(app.setDigitalOutput(LED_GREEN_PIN, true)).map_err(edgeError)?;
                log::info!("operit-esp32 online at http://{ip}/");
                Some(wifi)
            }
            Err(error) => {
                blockOn(app.setExpression("error")).map_err(edgeError)?;
                blockOn(app.setDigitalOutput(LED_RED_PIN, true)).map_err(edgeError)?;
                log::error!("operit-esp32 wifi failed: {}", error.message);
                None
            }
        }
    } else {
        blockOn(app.setExpression("online")).map_err(edgeError)?;
        log::warn!(
            "operit-esp32 starting without Wi-Fi; set OPERIT_WIFI_SSID to enable the home page"
        );
        None
    };
    let _home = if status.snapshot().ipv4.is_empty() {
        None
    } else {
        Some(Esp32WebHome::start(
            Arc::clone(&status),
            screenMirror,
            config.httpPort,
        )?)
    };

    use crate::ui::{HomeSurface, SwipeTracker};

    let mut swipe = SwipeTracker::new();
    loop {
        let point = match board.pollTouch() {
            Ok(point) => point,
            Err(error) => {
                log::warn!("operit-esp32 touch: {}", error.message);
                None
            }
        };
        if let Some(gesture) = swipe.onSample(point.map(|sample| (sample.x, sample.y))) {
            if app.handleGesture(gesture) {
                match app.surface() {
                    HomeSurface::PluginShelf => {
                        board.paintPluginShelf()?;
                        log::info!("operit-esp32 opened plugin shelf");
                    }
                    HomeSurface::Face => {
                        let expression = status.snapshot().expression;
                        blockOn(app.setExpression(&expression)).map_err(edgeError)?;
                        log::info!("operit-esp32 returned to face");
                    }
                }
            }
        }
        FreeRtos::delay_ms(20);
    }
}
