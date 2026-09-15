#![allow(non_snake_case)]

use std::sync::Arc;
use std::thread::JoinHandle;

use operit_edge_transport::{
    AuthenticatedLinkChannel, EdgeLinkServer, EdgePairingAuthority, EdgePairingStore,
    LinkChannel,
};
use operit_link::LinkDeviceInfo;
use operit_node_edge::EdgeNode;
use operit_host_api::{HostError, HostResult};
use tokio::runtime::Builder;

use crate::status::FirmwareStatus;

/// Runs the authenticated standard-Link listener on a dedicated ESP-IDF task.
pub struct Esp32EdgeLinkServer {
    _thread: JoinHandle<()>,
}

impl Esp32EdgeLinkServer {
    pub fn start(
        node: Arc<EdgeNode>,
        port: u16,
        token: String,
        status: Arc<FirmwareStatus>,
        store: Arc<dyn EdgePairingStore>,
    ) -> HostResult<Option<Self>> {
        if token.trim().is_empty() {
            log::warn!("Edge Link disabled: OPERIT_EDGE_TOKEN is not configured");
            return Ok(None);
        }
        let thread = std::thread::Builder::new()
            .name("operit-edge-link".to_string())
            .spawn(move || {
                let runtime = match Builder::new_current_thread().enable_all().build() {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        log::error!("Edge Link runtime: {error}");
                        return;
                    }
                };
                let authority = match EdgePairingAuthority::newWithStore(
                    token,
                    "esp32-edge".to_string(),
                    LinkDeviceInfo {
                        platform: "esp32".to_string(),
                        model: "ESP32-2432S028".to_string(),
                    },
                    store,
                    {
                        let status = Arc::clone(&status);
                        move |code| {
                            status.setPairingCode(code.clone());
                            log::info!("Edge Link pairing code: {code}");
                        }
                    },
                ) {
                    Ok(authority) => Arc::new(authority),
                    Err(error) => {
                        log::error!("Edge Link persistent store: {error}");
                        return;
                    }
                };
                runtime.block_on(async move {
                    let listener = match operit_edge_transport::tcp::TcpLinkChannel::bind(
                        &format!("0.0.0.0:{port}"),
                    )
                    .await
                    {
                        Ok(listener) => listener,
                        Err(error) => {
                            log::error!("Edge Link listener: {error}");
                            return;
                        }
                    };
                    log::info!("Edge Link listening on TCP port {port}");
                    loop {
                        let (stream, peer) = match listener.accept().await {
                            Ok(value) => value,
                            Err(error) => {
                                log::warn!("Edge Link accept: {error}");
                                continue;
                            }
                        };
                        log::info!("Edge Link connection from {peer}");
                        let channel = operit_edge_transport::tcp::TcpLinkChannel::fromStream(stream);
                        let authority = Arc::clone(&authority);
                        let node = Arc::clone(&node);
                        tokio::spawn(async move {
                            let first = match channel.receive().await {
                                Ok(Some(frame)) => frame,
                                Ok(None) => return,
                                Err(error) => {
                                    log::warn!("Edge Link first frame: {error}");
                                    return;
                                }
                            };
                            match &first.payload {
                                operit_link::LinkFramePayload::PairStart(request) => {
                                    match authority.pairFromStart(channel.clone(), request.clone()).await {
                                        Ok(session) => {
                                            let authenticated = AuthenticatedLinkChannel::new(channel, session);
                                            if let Err(error) = EdgeLinkServer::new(node, authenticated).run().await {
                                                log::warn!("Edge Link session: {error}");
                                            }
                                        }
                                        Err(error) => log::warn!("Edge Link pairing: {error}"),
                                    }
                                }
                                operit_link::LinkFramePayload::Authenticated { .. } => {
                                    match authority.authenticateFrame(&first) {
                                        Ok((session, inner)) => {
                                            let authenticated = AuthenticatedLinkChannel::new(channel, session);
                                            if let Err(error) = EdgeLinkServer::new(node, authenticated)
                                                .runWithFirstFrame(inner)
                                                .await
                                            {
                                                log::warn!("Edge Link resumed session: {error}");
                                            }
                                        }
                                        Err(error) => log::warn!("Edge Link authentication: {error}"),
                                    }
                                }
                                _ => log::warn!("Edge Link connection did not start with pairing or authentication"),
                            }
                        });
                    }
                });
            })
            .map_err(|error| HostError::new(format!("Edge Link thread: {error}")))?;
        Ok(Some(Self { _thread: thread }))
    }
}
