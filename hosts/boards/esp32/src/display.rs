#![allow(non_snake_case)]

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::{InputPin, Output, OutputPin, PinDriver};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::spi::{
    config::Config as SpiConfig, Dma, SpiAnyPins, SpiDeviceDriver, SpiDriver, SpiDriverConfig,
};
use esp_idf_hal::units::FromValueType;
use operit_host_api::{HostError, HostResult};

use crate::face::FaceRect;
use crate::pins::{logicalDisplaySize, madctlForRotation, DISPLAY_ROTATION_DEGREES};
use crate::robot_face::FaceCanvas;

const ILI9341_SWRESET: u8 = 0x01;
const ILI9341_SLPOUT: u8 = 0x11;
const ILI9341_NORON: u8 = 0x13;
const ILI9341_DISPON: u8 = 0x29;
const ILI9341_CASET: u8 = 0x2A;
const ILI9341_PASET: u8 = 0x2B;
const ILI9341_RAMWR: u8 = 0x2C;
const ILI9341_MADCTL: u8 = 0x36;
const ILI9341_PIXFMT: u8 = 0x3A;

/// Drives the ESP32-2432S028 ILI9341 panel over HSPI.
pub struct Esp32Ili9341 {
    spi: SpiDeviceDriver<'static, SpiDriver<'static>>,
    dc: PinDriver<'static, Output>,
    _backlight: PinDriver<'static, Output>,
    width: u16,
    height: u16,
}

impl Esp32Ili9341 {
    /// Initializes the ILI9341 at the firmware portrait rotation.
    pub fn new<SPI, SCLK, MOSI, MISO, CS, DC, BL>(
        spi: impl Peripheral<P = SPI> + 'static,
        sclk: SCLK,
        mosi: MOSI,
        miso: MISO,
        cs: CS,
        dc: DC,
        backlight: BL,
    ) -> HostResult<Self>
    where
        SPI: SpiAnyPins,
        SCLK: OutputPin + 'static,
        MOSI: OutputPin + 'static,
        MISO: InputPin + 'static,
        CS: OutputPin + 'static,
        DC: OutputPin + 'static,
        BL: OutputPin + 'static,
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
            &SpiConfig::new().baudrate(26.MHz().into()),
        )
        .map_err(|error| HostError::new(error.to_string()))?;
        let dc = PinDriver::output(dc).map_err(|error| HostError::new(error.to_string()))?;
        let mut backlight =
            PinDriver::output(backlight).map_err(|error| HostError::new(error.to_string()))?;
        backlight
            .set_high()
            .map_err(|error| HostError::new(error.to_string()))?;
        let (width, height) = logicalDisplaySize(DISPLAY_ROTATION_DEGREES);
        let mut display = Self {
            spi,
            dc,
            _backlight: backlight,
            width,
            height,
        };
        display.initialize()?;
        Ok(display)
    }

    /// Sends the ILI9341 power-up sequence and selects the firmware rotation.
    fn initialize(&mut self) -> HostResult<()> {
        self.writeCommand(ILI9341_SWRESET, &[])?;
        FreeRtos::delay_ms(150);
        self.writeCommand(ILI9341_SLPOUT, &[])?;
        FreeRtos::delay_ms(16);
        self.writeCommand(ILI9341_PIXFMT, &[0x55])?;
        self.writeCommand(ILI9341_MADCTL, &[madctlForRotation(DISPLAY_ROTATION_DEGREES)])?;
        self.writeCommand(ILI9341_NORON, &[])?;
        self.writeCommand(ILI9341_DISPON, &[])?;
        FreeRtos::delay_ms(16);
        Ok(())
    }

    /// Writes one ILI9341 command with optional data bytes.
    fn writeCommand(&mut self, command: u8, data: &[u8]) -> HostResult<()> {
        self.dc
            .set_low()
            .map_err(|error| HostError::new(error.to_string()))?;
        self.spi
            .write(&[command])
            .map_err(|error| HostError::new(error.to_string()))?;
        if !data.is_empty() {
            self.dc
                .set_high()
                .map_err(|error| HostError::new(error.to_string()))?;
            self.spi
                .write(data)
                .map_err(|error| HostError::new(error.to_string()))?;
        }
        Ok(())
    }

    /// Selects the ILI9341 column and row window for a later pixel burst.
    fn setWindow(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) -> HostResult<()> {
        self.writeCommand(
            ILI9341_CASET,
            &[
                (x0 >> 8) as u8,
                (x0 & 0xff) as u8,
                (x1 >> 8) as u8,
                (x1 & 0xff) as u8,
            ],
        )?;
        self.writeCommand(
            ILI9341_PASET,
            &[
                (y0 >> 8) as u8,
                (y0 & 0xff) as u8,
                (y1 >> 8) as u8,
                (y1 & 0xff) as u8,
            ],
        )
    }

    /// Fills the current window with one RGB565 color.
    fn fillWindow(&mut self, x0: u16, y0: u16, x1: u16, y1: u16, color: u16) -> HostResult<()> {
        if x0 > x1 || y0 > y1 {
            return Ok(());
        }
        self.setWindow(x0, y0, x1, y1)?;
        self.dc
            .set_low()
            .map_err(|error| HostError::new(error.to_string()))?;
        self.spi
            .write(&[ILI9341_RAMWR])
            .map_err(|error| HostError::new(error.to_string()))?;
        self.dc
            .set_high()
            .map_err(|error| HostError::new(error.to_string()))?;
        let pixel = [(color >> 8) as u8, (color & 0xff) as u8];
        let count = u32::from(x1 - x0 + 1) * u32::from(y1 - y0 + 1);
        let mut burst = [0u8; 128];
        for chunk in burst.chunks_mut(2) {
            chunk.copy_from_slice(&pixel);
        }
        let mut remaining = count;
        while remaining > 0 {
            let pixels = remaining.min(64);
            let bytes = (pixels * 2) as usize;
            self.spi
                .write(&burst[..bytes])
                .map_err(|error| HostError::new(error.to_string()))?;
            remaining -= pixels;
        }
        Ok(())
    }
}

impl FaceCanvas for Esp32Ili9341 {
    fn width(&self) -> u16 {
        self.width
    }

    fn height(&self) -> u16 {
        self.height
    }

    fn fill(&mut self, color: u16) -> HostResult<()> {
        if self.width == 0 || self.height == 0 {
            return Ok(());
        }
        self.fillWindow(0, 0, self.width - 1, self.height - 1, color)
    }

    fn fillRect(&mut self, rect: FaceRect, color: u16) -> HostResult<()> {
        if rect.width == 0 || rect.height == 0 {
            return Ok(());
        }
        let x1 = rect
            .x
            .saturating_add(rect.width)
            .saturating_sub(1)
            .min(self.width.saturating_sub(1));
        let y1 = rect
            .y
            .saturating_add(rect.height)
            .saturating_sub(1)
            .min(self.height.saturating_sub(1));
        if rect.x > x1 || rect.y > y1 {
            return Ok(());
        }
        self.fillWindow(rect.x, rect.y, x1, y1, color)
    }
}
