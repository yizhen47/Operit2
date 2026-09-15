#![allow(non_snake_case)]

use async_trait::async_trait;
use operit_link::{decodeLink, encodeLink, LinkFrame, LinkFramePayload};
use std::sync::Arc;

use crate::pairing::{signSession, EdgeSession};
use crate::LinkChannel;

/// Adds the existing Link-style session identity and HMAC to every frame.
pub struct AuthenticatedLinkChannel {
    inner: Arc<dyn LinkChannel>,
    session: EdgeSession,
}

impl AuthenticatedLinkChannel {
    pub fn new(inner: Arc<dyn LinkChannel>, session: EdgeSession) -> Arc<Self> {
        Arc::new(Self { inner, session })
    }

    fn wrap(&self, frame: LinkFrame) -> Result<LinkFrame, String> {
        let payloadBytes = encodeLink(&frame).map_err(|error| error.to_string())?;
        Ok(LinkFrame {
            messageId: frame.messageId,
            payload: LinkFramePayload::Authenticated {
                sessionId: self.session.sessionId.clone(),
                deviceId: self.session.deviceId.clone(),
                signature: signSession(&self.session.sessionSecret, &payloadBytes),
                payloadBytes,
            },
        })
    }

    fn unwrap(&self, frame: LinkFrame) -> Result<LinkFrame, String> {
        let LinkFramePayload::Authenticated {
            sessionId,
            deviceId,
            signature,
            payloadBytes,
        } = frame.payload
        else {
            return Err("Edge Link frame is not authenticated".to_string());
        };
        if sessionId != self.session.sessionId {
            return Err("Edge Link session id mismatch".to_string());
        }
        if deviceId != self.session.peerDeviceId {
            return Err("Edge Link device id mismatch".to_string());
        }
        if signSession(&self.session.sessionSecret, &payloadBytes) != signature {
            return Err("Edge Link signature mismatch".to_string());
        }
        decodeLink(&payloadBytes).map_err(|error| error.to_string())
    }
}

#[async_trait]
impl LinkChannel for AuthenticatedLinkChannel {
    async fn send(&self, frame: LinkFrame) -> Result<(), String> {
        self.inner.send(self.wrap(frame)?).await
    }

    async fn receive(&self) -> Result<Option<LinkFrame>, String> {
        let Some(frame) = self.inner.receive().await? else {
            return Ok(None);
        };
        self.unwrap(frame).map(Some)
    }

    async fn close(&self) {
        self.inner.close().await;
    }
}
