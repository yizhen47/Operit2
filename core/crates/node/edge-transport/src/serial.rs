#![allow(non_snake_case)]

use super::LinkChannel;
use async_trait::async_trait;
use operit_link::{decodeLink, encodeLink, LinkFrame};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::sync::Mutex;
use tokio_serial::{SerialPortBuilderExt, SerialStream};

const MAGIC: [u8; 4] = [0x4f, 0x50, 0x4c, 0x4b];
const VERSION: u8 = 1;
const HEADER_BYTES: usize = 13;
const MAX_LINK_FRAME_BYTES: usize = 256 * 1024;

/// Length/checksum framed serial carrier for standard Operit Link frames.
///
/// The magic prefix makes it possible to recover after ordinary UART logs or
/// a partial frame have appeared on the same physical line. Production ESP32
/// firmware should still use a dedicated UART for this carrier when possible.
pub struct SerialLinkChannel {
    reader: Mutex<ReadHalf<SerialStream>>,
    writer: Mutex<WriteHalf<SerialStream>>,
}

impl SerialLinkChannel {
    /// Opens a host serial port such as `COM27`.
    pub fn open(port: &str, baudRate: u32) -> Result<Arc<Self>, String> {
        let stream = tokio_serial::new(port, baudRate)
            .open_native_async()
            .map_err(|error| format!("open Edge serial port {port}: {error}"))?;
        Ok(Self::fromStream(stream))
    }

    pub fn fromStream(stream: SerialStream) -> Arc<Self> {
        let (reader, writer) = tokio::io::split(stream);
        Arc::new(Self {
            reader: Mutex::new(reader),
            writer: Mutex::new(writer),
        })
    }
}

#[async_trait]
impl LinkChannel for SerialLinkChannel {
    async fn send(&self, frame: LinkFrame) -> Result<(), String> {
        let payload = encodeLink(&frame).map_err(|error| error.to_string())?;
        if payload.is_empty() || payload.len() > MAX_LINK_FRAME_BYTES {
            return Err(format!("invalid Edge serial frame size: {}", payload.len()));
        }
        let length = u32::try_from(payload.len())
            .map_err(|_| "Edge serial frame exceeds u32 length".to_string())?;
        let checksum = crc32(&payload);
        let mut writer = self.writer.lock().await;
        writer.write_all(&MAGIC).await.map_err(|error| error.to_string())?;
        writer.write_all(&[VERSION]).await.map_err(|error| error.to_string())?;
        writer
            .write_all(&length.to_be_bytes())
            .await
            .map_err(|error| error.to_string())?;
        writer
            .write_all(&checksum.to_be_bytes())
            .await
            .map_err(|error| error.to_string())?;
        writer.write_all(&payload).await.map_err(|error| error.to_string())?;
        writer.flush().await.map_err(|error| error.to_string())
    }

    async fn receive(&self) -> Result<Option<LinkFrame>, String> {
        let mut reader = self.reader.lock().await;
        loop {
            if !findMagic(&mut *reader).await? {
                return Ok(None);
            }
            let mut version = [0u8; 1];
            match reader.read_exact(&mut version).await {
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
                Err(error) => return Err(error.to_string()),
            }
            if version[0] != VERSION {
                continue;
            }
            let mut header = [0u8; HEADER_BYTES - 5];
            reader.read_exact(&mut header).await.map_err(|error| error.to_string())?;
            let length = u32::from_be_bytes(header[0..4].try_into().unwrap()) as usize;
            let expectedChecksum = u32::from_be_bytes(header[4..8].try_into().unwrap());
            if length == 0 || length > MAX_LINK_FRAME_BYTES {
                continue;
            }
            let mut payload = vec![0u8; length];
            reader.read_exact(&mut payload).await.map_err(|error| error.to_string())?;
            if crc32(&payload) != expectedChecksum {
                continue;
            }
            return decodeLink(&payload)
                .map(Some)
                .map_err(|error| format!("decode Edge serial Link frame: {error}"));
        }
    }

    async fn close(&self) {
        let _ = self.writer.lock().await.shutdown().await;
    }
}

async fn findMagic<R: AsyncRead + Unpin>(reader: &mut R) -> Result<bool, String> {
    let mut window = [0u8; 4];
    let mut filled = 0usize;
    loop {
        let mut byte = [0u8; 1];
        match reader.read_exact(&mut byte).await {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(false),
            Err(error) => return Err(error.to_string()),
        }
        if filled < window.len() {
            window[filled] = byte[0];
            filled += 1;
        } else {
            window.copy_within(1.., 0);
            window[window.len() - 1] = byte[0];
        }
        if filled == window.len() && window == MAGIC {
            return Ok(true);
        }
    }
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::crc32;

    #[test]
    fn crcMatchesKnownVector() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
    }
}
