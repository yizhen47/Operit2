#![allow(non_snake_case)]

use crate::pins::{
    logicalDisplaySize, DISPLAY_ROTATION_DEGREES, PANEL_NATIVE_HEIGHT, PANEL_NATIVE_WIDTH,
};

/// Typical XPT2046 raw range on ESP32-2432S028.
pub const TOUCH_RAW_MIN: u16 = 200;

/// Typical XPT2046 raw range on ESP32-2432S028.
pub const TOUCH_RAW_MAX: u16 = 3900;

/// Maps one XPT2046 sample into the logical 240x320 face coordinate space.
pub fn mapRawToLogical(rawX: u16, rawY: u16) -> (u16, u16) {
    let (width, height) = logicalDisplaySize(DISPLAY_ROTATION_DEGREES);
    match DISPLAY_ROTATION_DEGREES % 360 {
        90 => (
            mapLinear(rawY, TOUCH_RAW_MIN, TOUCH_RAW_MAX, 0, width.saturating_sub(1)),
            mapLinear(rawX, TOUCH_RAW_MAX, TOUCH_RAW_MIN, 0, height.saturating_sub(1)),
        ),
        180 => (
            mapLinear(rawX, TOUCH_RAW_MAX, TOUCH_RAW_MIN, 0, width.saturating_sub(1)),
            mapLinear(rawY, TOUCH_RAW_MAX, TOUCH_RAW_MIN, 0, height.saturating_sub(1)),
        ),
        270 => (
            mapLinear(rawY, TOUCH_RAW_MAX, TOUCH_RAW_MIN, 0, width.saturating_sub(1)),
            mapLinear(rawX, TOUCH_RAW_MAX, TOUCH_RAW_MIN, 0, height.saturating_sub(1)),
        ),
        _ => (
            mapLinear(rawX, TOUCH_RAW_MIN, TOUCH_RAW_MAX, 0, PANEL_NATIVE_HEIGHT.saturating_sub(1)),
            mapLinear(rawY, TOUCH_RAW_MIN, TOUCH_RAW_MAX, 0, PANEL_NATIVE_WIDTH.saturating_sub(1)),
        ),
    }
}

/// Linearly maps `value` from `[inA, inB]` into `[outA, outB]`.
fn mapLinear(value: u16, inA: u16, inB: u16, outA: u16, outB: u16) -> u16 {
    let clamped = if inA <= inB {
        value.clamp(inA, inB)
    } else {
        value.clamp(inB, inA)
    };
    let inRange = i32::from(inB) - i32::from(inA);
    if inRange == 0 {
        return outA;
    }
    let mapped = i32::from(outA)
        + (i32::from(clamped) - i32::from(inA)) * (i32::from(outB) - i32::from(outA)) / inRange;
    let outMin = i32::from(outA.min(outB));
    let outMax = i32::from(outA.max(outB));
    mapped.clamp(outMin, outMax) as u16
}

/// One calibrated touch sample in logical display pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TouchPoint {
    pub x: u16,
    pub y: u16,
}

#[cfg(target_os = "espidf")]
mod driver {
    use esp_idf_hal::gpio::{Input, InputPin, OutputPin, PinDriver, Pull};
    use esp_idf_hal::spi::{
        config::Config as SpiConfig, Dma, SpiAnyPins, SpiDeviceDriver, SpiDriver, SpiDriverConfig,
    };
    use esp_idf_hal::units::FromValueType;
    use operit_host_api::{HostError, HostResult};

    use super::{mapRawToLogical, TouchPoint};

    const CMD_X: u8 = 0xd0;
    const CMD_Y: u8 = 0x90;
    const CMD_Z1: u8 = 0xb0;
    const Z_MIN: u16 = 80;

    /// Reads the ESP32-2432S028 XPT2046 over VSPI.
    pub struct Esp32Touch {
        spi: SpiDeviceDriver<'static, SpiDriver<'static>>,
        irq: PinDriver<'static, Input>,
    }

    impl Esp32Touch {
        /// Takes the touch SPI and IRQ pins out of ESP-IDF peripherals.
        pub fn new<SPI, SCLK, MOSI, MISO, CS, IRQ>(
            spi: SPI,
            sclk: SCLK,
            mosi: MOSI,
            miso: MISO,
            cs: CS,
            irq: IRQ,
        ) -> HostResult<Self>
        where
            SPI: SpiAnyPins + 'static,
            SCLK: OutputPin + 'static,
            MOSI: OutputPin + 'static,
            MISO: InputPin + 'static,
            CS: OutputPin + 'static,
            IRQ: InputPin + 'static,
        {
            let driver = SpiDriver::new(
                spi,
                sclk,
                mosi,
                Some(miso),
                &SpiDriverConfig::new().dma(Dma::Disabled),
            )
            .map_err(|error| HostError::new(error.to_string()))?;
            let spi = SpiDeviceDriver::new(
                driver,
                Some(cs),
                &SpiConfig::new().baudrate(2.MHz().into()),
            )
            .map_err(|error| HostError::new(error.to_string()))?;
            let irq = PinDriver::input(irq, Pull::Floating)
                .map_err(|error| HostError::new(error.to_string()))?;
            Ok(Self { spi, irq })
        }

        /// Returns one logical point while the panel is pressed.
        pub fn poll(&mut self) -> HostResult<Option<TouchPoint>> {
            if self.irq.is_high() {
                return Ok(None);
            }
            let z = self.read12(CMD_Z1)?;
            if z < Z_MIN {
                return Ok(None);
            }
            let rawX = (self.read12(CMD_X)? + self.read12(CMD_X)?) / 2;
            let rawY = (self.read12(CMD_Y)? + self.read12(CMD_Y)?) / 2;
            let (x, y) = mapRawToLogical(rawX, rawY);
            Ok(Some(TouchPoint { x, y }))
        }

        /// Reads one 12-bit XPT2046 conversion in a single CS transaction.
        fn read12(&mut self, command: u8) -> HostResult<u16> {
            let mut buffer = [command, 0, 0];
            self.spi
                .transfer_in_place(&mut buffer)
                .map_err(|error| HostError::new(error.to_string()))?;
            Ok((u16::from(buffer[1]) << 8 | u16::from(buffer[2])) >> 3)
        }
    }
}

#[cfg(target_os = "espidf")]
pub use driver::Esp32Touch;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portraitRotationMapsCenterNearScreenCenter() {
        let (x, y) = mapRawToLogical(2050, 2050);
        assert!(x > 80 && x < 160, "x={x}");
        assert!(y > 120 && y < 200, "y={y}");
    }

    #[test]
    fn portraitRotationKeepsSamplesOnScreen() {
        let (x, y) = mapRawToLogical(TOUCH_RAW_MIN, TOUCH_RAW_MAX);
        assert!(x < 240);
        assert!(y < 320);
        let (x, y) = mapRawToLogical(TOUCH_RAW_MAX, TOUCH_RAW_MIN);
        assert!(x < 240);
        assert!(y < 320);
    }
}
