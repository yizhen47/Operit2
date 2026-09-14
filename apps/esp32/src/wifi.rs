#![allow(non_snake_case)]

use std::net::Ipv4Addr;

use esp_idf_hal::modem::Modem;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi};
use operit_host_api::{HostError, HostResult};

use crate::config::Esp32FirmwareConfig;

/// Holds the ESP-IDF station client for the firmware lifetime.
pub struct Esp32Wifi {
    wifi: BlockingWifi<EspWifi<'static>>,
}

impl Esp32Wifi {
    /// Connects to the compile-time station network and waits for IPv4.
    pub fn connect(modem: Modem<'static>, config: &Esp32FirmwareConfig) -> HostResult<Self> {
        let sysLoop = EspSystemEventLoop::take()
            .map_err(|error| HostError::new(format!("event loop: {error}")))?;
        let nvs = EspDefaultNvsPartition::take()
            .map_err(|error| HostError::new(format!("nvs: {error}")))?;
        let mut wifi = BlockingWifi::wrap(
            EspWifi::new(modem, sysLoop.clone(), Some(nvs))
                .map_err(|error| HostError::new(format!("wifi driver: {error}")))?,
            sysLoop,
        )
        .map_err(|error| HostError::new(format!("wifi wrap: {error}")))?;
        let mut client = ClientConfiguration::default();
        client.ssid.clear();
        client
            .ssid
            .push_str(&config.wifiSsid)
            .map_err(|_| HostError::new("Wi-Fi SSID exceeds the ESP-IDF station field"))?;
        client.password.clear();
        client
            .password
            .push_str(&config.wifiPassword)
            .map_err(|_| HostError::new("Wi-Fi password exceeds the ESP-IDF station field"))?;
        client.auth_method = if config.wifiPassword.is_empty() {
            AuthMethod::None
        } else {
            AuthMethod::WPA2Personal
        };
        wifi.set_configuration(&Configuration::Client(client))
            .map_err(|error| HostError::new(format!("wifi config: {error}")))?;
        wifi.start()
            .map_err(|error| HostError::new(format!("wifi start: {error}")))?;
        wifi.connect()
            .map_err(|error| HostError::new(format!("wifi connect: {error}")))?;
        wifi.wait_netif_up()
            .map_err(|error| HostError::new(format!("wifi netif: {error}")))?;
        Ok(Self { wifi })
    }

    /// Returns the station IPv4 address after association.
    pub fn ipv4(&self) -> HostResult<Ipv4Addr> {
        let info = self
            .wifi
            .wifi()
            .sta_netif()
            .get_ip_info()
            .map_err(|error| HostError::new(format!("wifi ip: {error}")))?;
        Ok(info.ip)
    }
}
