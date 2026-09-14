#![allow(non_snake_case)]

/// Board identifier for the ESP32-2432S028 Edge firmware.
pub const ESP32_2432S028_BOARD_ID: &str = "ESP32-2432S028";

/// Display name for the ESP32-2432S028 Edge firmware.
pub const ESP32_2432S028_BOARD_DISPLAY_NAME: &str = "ESP32-2432S028";

/// ILI9341 native panel width in pixels.
pub const PANEL_NATIVE_WIDTH: u16 = 240;

/// ILI9341 native panel height in pixels.
pub const PANEL_NATIVE_HEIGHT: u16 = 320;

/// Clockwise display rotation in degrees from the CircuitPython landscape default.
///
/// `270` is 90 degrees counter-clockwise from 320x240 and produces a 240x320
/// portrait face layout.
pub const DISPLAY_ROTATION_DEGREES: u16 = 270;

/// TFT SPI clock pin.
pub const TFT_SCLK_PIN: u8 = 14;

/// TFT SPI MOSI pin.
pub const TFT_MOSI_PIN: u8 = 13;

/// TFT SPI MISO pin.
pub const TFT_MISO_PIN: u8 = 12;

/// TFT chip-select pin.
pub const TFT_CS_PIN: u8 = 15;

/// TFT data/command pin.
pub const TFT_DC_PIN: u8 = 2;

/// TFT backlight pin.
pub const TFT_BACKLIGHT_PIN: u8 = 21;

/// Resistive-touch SPI clock pin.
pub const TOUCH_SCLK_PIN: u8 = 25;

/// Resistive-touch SPI MOSI pin.
pub const TOUCH_MOSI_PIN: u8 = 32;

/// Resistive-touch SPI MISO pin.
pub const TOUCH_MISO_PIN: u8 = 39;

/// Resistive-touch chip-select pin.
pub const TOUCH_CS_PIN: u8 = 33;

/// Resistive-touch interrupt pin. Active-low while pressed.
pub const TOUCH_INT_PIN: u8 = 36;

/// Active-low red status LED pin.
pub const LED_RED_PIN: u8 = 4;

/// Active-low green status LED pin.
pub const LED_GREEN_PIN: u8 = 16;

/// Active-low blue status LED pin.
pub const LED_BLUE_PIN: u8 = 17;

/// SD-card chip-select pin. Reserved for a later Host surface.
pub const SD_CS_PIN: u8 = 5;

/// ILI9341 BGR bit for MADCTL.
const MADCTL_BGR: u8 = 0x08;

/// ILI9341 MX bit for MADCTL.
const MADCTL_MX: u8 = 0x40;

/// ILI9341 MY bit for MADCTL.
const MADCTL_MY: u8 = 0x80;

/// ILI9341 MV bit for MADCTL.
const MADCTL_MV: u8 = 0x20;

/// Returns the logical face size for one clockwise rotation in degrees.
pub fn logicalDisplaySize(rotationDegrees: u16) -> (u16, u16) {
    match rotationDegrees % 360 {
        90 | 270 => (PANEL_NATIVE_WIDTH, PANEL_NATIVE_HEIGHT),
        _ => (PANEL_NATIVE_HEIGHT, PANEL_NATIVE_WIDTH),
    }
}

/// Returns the ILI9341 MADCTL value for one clockwise rotation in degrees.
pub fn madctlForRotation(rotationDegrees: u16) -> u8 {
    match rotationDegrees % 360 {
        0 => MADCTL_BGR | MADCTL_MX | MADCTL_MV,
        90 => MADCTL_BGR | MADCTL_MX,
        180 => MADCTL_BGR | MADCTL_MY,
        270 => MADCTL_BGR,
        _ => madctlForRotation(DISPLAY_ROTATION_DEGREES),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies the firmware portrait rotation reports 240x320.
    #[test]
    fn portraitRotationUsesNativePanelSize() {
        assert_eq!(
            logicalDisplaySize(DISPLAY_ROTATION_DEGREES),
            (PANEL_NATIVE_WIDTH, PANEL_NATIVE_HEIGHT)
        );
        assert_eq!(madctlForRotation(270), MADCTL_BGR);
    }

    /// Verifies the CircuitPython landscape default reports 320x240.
    #[test]
    fn landscapeRotationSwapsPanelAxes() {
        assert_eq!(
            logicalDisplaySize(0),
            (PANEL_NATIVE_HEIGHT, PANEL_NATIVE_WIDTH)
        );
    }
}
