#![allow(non_snake_case)]

mod app;
mod config;
mod status;

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
    use std::sync::Arc;

    use esp_idf_hal::delay::FreeRtos;
    use esp_idf_hal::peripherals::Peripherals;
    use operit_board_esp32::{Esp32Board, INITIAL_EXPRESSION, LED_GREEN_PIN, LED_RED_PIN};
    use operit_host_api::HostError;
    use operit_node_edge::{createDeviceIoService, createRobotFaceService, EdgeNode};
    use operit_proxy_edge::{EdgeProxy, EdgeProxyError};

    fn edgeError(error: EdgeProxyError) -> HostError {
        HostError::new(error.to_string())
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
    let board = Esp32Board::new(
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
    )?;
    let hostManager = board.installIntoHostManager();
    let edgeNode = EdgeNode::new(createDeviceIoService(hostManager.clone()))
        .withRobotFaceService(createRobotFaceService(hostManager));
    let edgeProxy = EdgeProxy::new(edgeNode);
    let status = Arc::new(FirmwareStatus::new(INITIAL_EXPRESSION));
    let mut app = Esp32App::new(edgeProxy, Arc::clone(&status));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|error| HostError::new(error.to_string()))?;

    runtime
        .block_on(app.setExpression("booting"))
        .map_err(edgeError)?;
    let _wifi = if config.hasWifi() {
        match Esp32Wifi::connect(modem, &config) {
            Ok(wifi) => {
                let ip = wifi.ipv4()?;
                status.setWifiSsid(config.wifiSsid.clone());
                status.setIpv4(ip.to_string());
                runtime
                    .block_on(app.setExpression("online"))
                    .map_err(edgeError)?;
                runtime
                    .block_on(app.setDigitalOutput(LED_GREEN_PIN, true))
                    .map_err(edgeError)?;
                log::info!("operit-esp32 online at http://{ip}/");
                Some(wifi)
            }
            Err(error) => {
                runtime
                    .block_on(app.setExpression("error"))
                    .map_err(edgeError)?;
                runtime
                    .block_on(app.setDigitalOutput(LED_RED_PIN, true))
                    .map_err(edgeError)?;
                log::error!("operit-esp32 wifi failed: {}", error.message);
                None
            }
        }
    } else {
        runtime
            .block_on(app.setExpression("online"))
            .map_err(edgeError)?;
        log::warn!(
            "operit-esp32 starting without Wi-Fi; set OPERIT_WIFI_SSID to enable the home page"
        );
        None
    };
    let _home = if status.snapshot().ipv4.is_empty() {
        None
    } else {
        Some(Esp32WebHome::start(Arc::clone(&status), config.httpPort)?)
    };

    loop {
        FreeRtos::delay_ms(1000);
    }
}
