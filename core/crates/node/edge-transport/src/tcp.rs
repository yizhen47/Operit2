#![allow(non_snake_case)]

use super::LinkChannel;
use async_trait::async_trait;
use operit_link::{decodeLink, encodeLink, LinkFrame};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

const MAX_LINK_FRAME_BYTES: usize = 1024 * 1024;

/// Length-prefixed TCP carrier for standard Link frames.
pub struct TcpLinkChannel {
    reader: Mutex<ReadHalf<TcpStream>>,
    writer: Mutex<WriteHalf<TcpStream>>,
}

impl TcpLinkChannel {
    pub fn fromStream(stream: TcpStream) -> Arc<Self> {
        let (reader, writer) = tokio::io::split(stream);
        Arc::new(Self {
            reader: Mutex::new(reader),
            writer: Mutex::new(writer),
        })
    }

    pub async fn connect(address: &str) -> Result<Arc<Self>, String> {
        let stream = TcpStream::connect(address)
            .await
            .map_err(|error| format!("Edge TCP connect {address}: {error}"))?;
        Ok(Self::fromStream(stream))
    }

    pub async fn bind(address: &str) -> Result<TcpListener, String> {
        TcpListener::bind(address)
            .await
            .map_err(|error| format!("Edge TCP bind {address}: {error}"))
    }
}

#[async_trait]
impl LinkChannel for TcpLinkChannel {
    async fn send(&self, frame: LinkFrame) -> Result<(), String> {
        let payload = encodeLink(&frame).map_err(|error| error.to_string())?;
        let length = u32::try_from(payload.len())
            .map_err(|_| "Edge Link frame exceeds u32 length".to_string())?;
        let mut writer = self.writer.lock().await;
        writer
            .write_all(&length.to_be_bytes())
            .await
            .map_err(|error| error.to_string())?;
        writer
            .write_all(&payload)
            .await
            .map_err(|error| error.to_string())?;
        writer.flush().await.map_err(|error| error.to_string())
    }

    async fn receive(&self) -> Result<Option<LinkFrame>, String> {
        let mut reader = self.reader.lock().await;
        let mut lengthBytes = [0u8; 4];
        match reader.read_exact(&mut lengthBytes).await {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(error) => return Err(error.to_string()),
        }
        let length = u32::from_be_bytes(lengthBytes) as usize;
        if length == 0 || length > MAX_LINK_FRAME_BYTES {
            return Err(format!("invalid Edge Link frame length: {length}"));
        }
        let mut payload = vec![0u8; length];
        reader
            .read_exact(&mut payload)
            .await
            .map_err(|error| error.to_string())?;
        decodeLink(&payload).map(Some).map_err(|error| error.to_string())
    }

    async fn close(&self) {
        let _ = self.writer.lock().await.shutdown().await;
    }
}
