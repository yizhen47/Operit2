#![allow(non_snake_case)]

use std::sync::Mutex;

use operit_board_esp32::ESP32_2432S028_BOARD_ID;

/// Snapshot published by the firmware HTTP home page.
#[derive(Clone, Debug)]
pub struct FirmwareStatusSnapshot {
    pub boardId: String,
    pub expression: String,
    pub ipv4: String,
    pub wifiSsid: String,
    pub pairingCode: String,
}

/// Shared firmware status consumed by the local HTTP home page.
pub struct FirmwareStatus {
    boardId: String,
    expression: Mutex<String>,
    ipv4: Mutex<String>,
    wifiSsid: Mutex<String>,
    pairingCode: Mutex<String>,
}

impl FirmwareStatus {
    /// Creates firmware status with no network address yet.
    pub fn new(expression: impl Into<String>) -> Self {
        Self {
            boardId: ESP32_2432S028_BOARD_ID.to_string(),
            expression: Mutex::new(expression.into()),
            ipv4: Mutex::new(String::new()),
            wifiSsid: Mutex::new(String::new()),
            pairingCode: Mutex::new(String::new()),
        }
    }

    /// Records the current robot face expression.
    pub fn setExpression(&self, expression: impl Into<String>) {
        if let Ok(mut current) = self.expression.lock() {
            *current = expression.into();
        }
    }

    /// Records the current station IPv4 address.
    pub fn setIpv4(&self, ipv4: impl Into<String>) {
        if let Ok(mut current) = self.ipv4.lock() {
            *current = ipv4.into();
        }
    }

    /// Records the associated Wi-Fi SSID without storing the password.
    pub fn setWifiSsid(&self, ssid: impl Into<String>) {
        if let Ok(mut current) = self.wifiSsid.lock() {
            *current = ssid.into();
        }
    }

    /// Publishes the one-time Edge pairing code while a pairing is pending.
    pub fn setPairingCode(&self, code: impl Into<String>) {
        if let Ok(mut current) = self.pairingCode.lock() {
            *current = code.into();
        }
    }

    /// Returns a copy of the values shown on the firmware home page.
    pub fn snapshot(&self) -> FirmwareStatusSnapshot {
        FirmwareStatusSnapshot {
            boardId: self.boardId.clone(),
            expression: self
                .expression
                .lock()
                .map(|value| value.clone())
                .unwrap_or_default(),
            ipv4: self
                .ipv4
                .lock()
                .map(|value| value.clone())
                .unwrap_or_default(),
            wifiSsid: self
                .wifiSsid
                .lock()
                .map(|value| value.clone())
                .unwrap_or_default(),
            pairingCode: self
                .pairingCode
                .lock()
                .map(|value| value.clone())
                .unwrap_or_default(),
        }
    }
}

/// Renders the firmware home page HTML for one status snapshot.
pub fn renderHomePage(snapshot: &FirmwareStatusSnapshot) -> String {
    let ip = if snapshot.ipv4.is_empty() {
        "unavailable"
    } else {
        snapshot.ipv4.as_str()
    };
    let ssid = if snapshot.wifiSsid.is_empty() {
        "unconfigured"
    } else {
        snapshot.wifiSsid.as_str()
    };
    format!(
        "<!DOCTYPE html>\
<html lang=\"zh-CN\">\
<head><meta charset=\"utf-8\"><title>Operit2 ESP32</title></head>\
<body>\
<h1>Operit2 Edge</h1>\
<p>Board: {board}</p>\
<p>Expression: {expression}</p>\
<p>Wi-Fi: {ssid}</p>\
<p>IP: {ip}</p>\
<p>Pairing code: {pairing}</p>\
<p><a href=\"/screen\">打开屏幕实时预览</a></p>\
<p>This node is an Edge capability device, not a full CoreNode.</p>\
</body>\
</html>",
        board = htmlEscape(&snapshot.boardId),
        expression = htmlEscape(&snapshot.expression),
        ssid = htmlEscape(ssid),
        ip = htmlEscape(ip),
        pairing = htmlEscape(&snapshot.pairingCode),
    )
}

/// Renders firmware status as JSON for machine clients.
pub fn renderStatusJson(snapshot: &FirmwareStatusSnapshot) -> String {
    format!(
        "{{\"boardId\":\"{}\",\"expression\":\"{}\",\"wifiSsid\":\"{}\",\"ipv4\":\"{}\",\"pairingCode\":\"{}\"}}",
        jsonEscape(&snapshot.boardId),
        jsonEscape(&snapshot.expression),
        jsonEscape(&snapshot.wifiSsid),
        jsonEscape(&snapshot.ipv4),
        jsonEscape(&snapshot.pairingCode)
    )
}

/// Escapes HTML text content.
fn htmlEscape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Escapes a JSON string value.
fn jsonEscape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies the home page includes board identity and expression.
    #[test]
    fn homePageIncludesBoardState() {
        let status = FirmwareStatus::new("booting");
        status.setWifiSsid("test-net");
        status.setIpv4("192.168.1.104");
        let html = renderHomePage(&status.snapshot());
        assert!(html.contains("ESP32-2432S028"));
        assert!(html.contains("booting"));
        assert!(html.contains("192.168.1.104"));
        assert!(!html.contains("password"));
    }
}
