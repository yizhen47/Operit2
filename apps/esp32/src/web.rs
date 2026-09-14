#![allow(non_snake_case)]

use std::sync::Arc;

use esp_idf_svc::http::server::{Configuration as HttpConfig, EspHttpServer};
use esp_idf_svc::http::Method;
use esp_idf_svc::io::Write;
use operit_board_esp32::{Esp32ScreenMirror, Esp32ScreenMirrorRect};
use operit_host_api::{HostError, HostResult};

use crate::status::{renderHomePage, renderStatusJson, FirmwareStatus};

/// Holds the firmware HTTP server for the process lifetime.
pub struct Esp32WebHome {
    _server: EspHttpServer<'static>,
}

impl Esp32WebHome {
    /// Serves status and a browser-based live preview of the display.
    pub fn start(
        status: Arc<FirmwareStatus>,
        screenMirror: Arc<Esp32ScreenMirror>,
        httpPort: u16,
    ) -> HostResult<Self> {
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
        server
            .fn_handler("/screen", Method::Get, move |request| {
                let mut response = request.into_response(
                    200,
                    Some("OK"),
                    &[("Content-Type", "text/html; charset=utf-8")],
                )?;
                response.write_all(renderScreenPage().as_bytes())?;
                Ok::<(), esp_idf_svc::io::EspIOError>(())
            })
            .map_err(|error| HostError::new(format!("http /screen: {error}")))?;
        server
            .fn_handler("/screen.bmp", Method::Get, move |request| {
                let mut response = request.into_response(
                    200,
                    Some("OK"),
                    &[
                        ("Content-Type", "image/bmp"),
                        ("Cache-Control", "no-store, no-cache, must-revalidate"),
                    ],
                )?;
                let (width, height) = screenMirror.dimensions();
                screenMirror.withState(|background, rects| {
                    writeScreenBmp(&mut response, background, rects, width, height)
                })?;
                Ok::<(), esp_idf_svc::io::EspIOError>(())
            })
            .map_err(|error| HostError::new(format!("http /screen.bmp: {error}")))?;
        Ok(Self { _server: server })
    }
}

/// Renders the browser page that refreshes the display preview after each image loads.
fn renderScreenPage() -> &'static str {
    "<!DOCTYPE html>\
<html lang=\"zh-CN\">\
<head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>ESP32 屏幕预览</title>\
<style>body{margin:0;padding:20px;background:#10131a;color:#e8edf5;font-family:system-ui,sans-serif;text-align:center}img{display:block;width:min(480px,90vw);height:auto;margin:16px auto;border:1px solid #394354;image-rendering:pixelated;background:#080c18}p{color:#aeb9ca}</style></head>\
<body><h1>ESP32 屏幕预览</h1><p id=\"state\">正在连接...</p><img id=\"screen\" alt=\"ESP32 屏幕\">\
<script>const image=document.getElementById('screen'),state=document.getElementById('state');function refresh(){image.src='/screen.bmp?t='+Date.now();}image.onload=()=>{state.textContent='已连接 · '+new Date().toLocaleTimeString();setTimeout(refresh,250)};image.onerror=()=>{state.textContent='读取屏幕失败，正在重试...';setTimeout(refresh,1000)};refresh();</script>\
</body></html>"
}

/// Writes the current RGB565 framebuffer as a compact 16-bit RGB565 BMP.
fn writeScreenBmp<W: Write>(
    response: &mut W,
    background: u16,
    rects: &[Esp32ScreenMirrorRect],
    width: u16,
    height: u16,
) -> Result<(), W::Error> {
    let rowBytes = usize::from(width) * 2;
    let imageSize = rowBytes * usize::from(height);
    let fileSize = 66usize + imageSize;
    let mut header = [0u8; 66];
    header[0] = b'B';
    header[1] = b'M';
    header[2..6].copy_from_slice(&(fileSize as u32).to_le_bytes());
    header[10..14].copy_from_slice(&66u32.to_le_bytes());
    header[14..18].copy_from_slice(&40u32.to_le_bytes());
    header[18..22].copy_from_slice(&i32::from(width).to_le_bytes());
    header[22..26].copy_from_slice(&i32::from(height).to_le_bytes());
    header[26..28].copy_from_slice(&1u16.to_le_bytes());
    header[28..30].copy_from_slice(&16u16.to_le_bytes());
    header[30..34].copy_from_slice(&3u32.to_le_bytes());
    header[34..38].copy_from_slice(&(imageSize as u32).to_le_bytes());
    header[38..42].copy_from_slice(&2835i32.to_le_bytes());
    header[42..46].copy_from_slice(&2835i32.to_le_bytes());
    header[54..58].copy_from_slice(&0xF800u32.to_le_bytes());
    header[58..62].copy_from_slice(&0x07E0u32.to_le_bytes());
    header[62..66].copy_from_slice(&0x001Fu32.to_le_bytes());
    response.write_all(&header)?;

    let mut row = vec![0u8; rowBytes];
    for y in (0..usize::from(height)).rev() {
        for x in 0..usize::from(width) {
            let pixel = mirroredPixel(background, rects, x as u16, y as u16);
            row[x * 2] = pixel as u8;
            row[x * 2 + 1] = (pixel >> 8) as u8;
        }
        response.write_all(&row)?;
    }
    Ok(())
}

/// Resolves one pixel by replaying the compact ordered drawing commands.
fn mirroredPixel(background: u16, rects: &[Esp32ScreenMirrorRect], x: u16, y: u16) -> u16 {
    let mut color = background;
    for command in rects {
        let right = command.rect.x.saturating_add(command.rect.width);
        let bottom = command.rect.y.saturating_add(command.rect.height);
        if x >= command.rect.x && x < right && y >= command.rect.y && y < bottom {
            color = command.color;
        }
    }
    color
}
