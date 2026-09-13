#![allow(non_snake_case)]

/// Compile-time Wi-Fi and HTTP settings for the ESP32-2432S028 firmware.
#[derive(Clone, Debug)]
pub struct Esp32FirmwareConfig {
    pub wifiSsid: String,
    pub wifiPassword: String,
    pub httpPort: u16,
}

impl Esp32FirmwareConfig {
    /// Reads firmware settings from `OPERIT_WIFI_SSID` and `OPERIT_WIFI_PASSWORD`.
    pub fn fromEnv() -> Self {
        Self {
            wifiSsid: option_env!("OPERIT_WIFI_SSID")
                .unwrap_or("")
                .to_string(),
            wifiPassword: option_env!("OPERIT_WIFI_PASSWORD")
                .unwrap_or("")
                .to_string(),
            httpPort: 80,
        }
    }

    /// Returns whether Wi-Fi station credentials were supplied at build time.
    pub fn hasWifi(&self) -> bool {
        !self.wifiSsid.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies empty compile-time credentials disable Wi-Fi startup.
    #[test]
    fn emptyCredentialsDisableWifi() {
        let config = Esp32FirmwareConfig {
            wifiSsid: String::new(),
            wifiPassword: String::new(),
            httpPort: 80,
        };
        assert!(!config.hasWifi());
    }
}
