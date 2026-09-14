#![allow(non_snake_case)]

use std::sync::Arc;

use esp_idf_svc::http::server::{Configuration as HttpConfig, EspHttpServer};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;
use operit_host_api::{HostError, HostResult};

use crate::status::{renderHomePage, renderStatusJson, FirmwareStatus};

/// Holds the firmware HTTP server for the process lifetime.
pub struct Esp32WebHome {
    _server: EspHttpServer<'static>,
}

impl Esp32WebHome {
    /// Serves `/` and `/status.json` from the shared firmware status.
    pub fn start(status: Arc<FirmwareStatus>, httpPort: u16) -> HostResult<Self> {
        let mut server = EspHttpServer::new(&HttpConfig {
            http_port: httpPort,
            ..Default::default()
        })
        .map_err(|error| HostError::new(format!("http server: {error}")))?;
        let homeStatus = Arc::clone(&status);
        server
            .fn_handler("/", Method::Get, move |request| {
                let body = renderHomePage(&homeStatus.snapshot());
                let mut response = request.into_ok_response()?;
                response.write_all(body.as_bytes())?;
                Ok::<(), esp_idf_svc::io::EspIOError>(())
            })
            .map_err(|error| HostError::new(format!("http /: {error}")))?;
        server
            .fn_handler("/status.json", Method::Get, move |request| {
                let body = renderStatusJson(&status.snapshot());
                let mut response = request.into_response(
                    200,
                    Some("OK"),
                    &[("Content-Type", "application/json; charset=utf-8")],
                )?;
                response.write_all(body.as_bytes())?;
                Ok::<(), esp_idf_svc::io::EspIOError>(())
            })
            .map_err(|error| HostError::new(format!("http /status.json: {error}")))?;
        Ok(Self { _server: server })
    }
}
