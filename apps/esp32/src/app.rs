#![allow(non_snake_case)]

use std::sync::Arc;

use operit_host_api::{DeviceDigitalOutputState, RobotFaceState};
use operit_proxy_edge::{EdgeDeviceIoClient, EdgeProxyError, EdgeRobotFaceClient};

use crate::status::FirmwareStatus;

/// Owns ESP32-2432S028 device behavior on top of the Edge Proxy contract.
pub struct Esp32App<C> {
    edgeClient: C,
    status: Arc<FirmwareStatus>,
}

impl<C> Esp32App<C> {
    /// Creates the firmware app around one Edge proxy and shared HTTP status.
    pub fn new(edgeClient: C, status: Arc<FirmwareStatus>) -> Self {
        Self { edgeClient, status }
    }
}

impl<C> Esp32App<C>
where
    C: EdgeRobotFaceClient,
{
    /// Publishes one robot expression through Edge and mirrors it to HTTP status.
    pub async fn setExpression(
        &mut self,
        expression: &str,
    ) -> Result<RobotFaceState, EdgeProxyError> {
        let state = self
            .edgeClient
            .setExpression(expression.to_string())
            .await?;
        self.status.setExpression(state.expression.clone());
        Ok(state)
    }
}

impl<C> Esp32App<C>
where
    C: EdgeDeviceIoClient,
{
    /// Writes one board digital output through Edge.
    pub async fn setDigitalOutput(
        &mut self,
        pin: u8,
        level: bool,
    ) -> Result<DeviceDigitalOutputState, EdgeProxyError> {
        self.edgeClient.setDigitalOutput(pin, level).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use operit_host_api::DeviceDigitalOutputState;
    use operit_proxy_edge::{EdgeDeviceIoClient, EdgeRobotFaceStateStream, EdgeStateStream};

    struct TestEdgeClient {
        expression: String,
    }

    #[async_trait(?Send)]
    impl EdgeRobotFaceClient for TestEdgeClient {
        async fn setExpression(
            &mut self,
            expression: String,
        ) -> Result<RobotFaceState, EdgeProxyError> {
            self.expression = expression.clone();
            Ok(RobotFaceState { expression })
        }

        async fn getExpression(&mut self) -> Result<RobotFaceState, EdgeProxyError> {
            Ok(RobotFaceState {
                expression: self.expression.clone(),
            })
        }

        async fn watchExpression(&mut self) -> Result<EdgeRobotFaceStateStream, EdgeProxyError> {
            Err(EdgeProxyError {
                code: "UNSUPPORTED".to_string(),
                message: "watch is unused in firmware unit tests".to_string(),
            })
        }
    }

    #[async_trait(?Send)]
    impl EdgeDeviceIoClient for TestEdgeClient {
        async fn setDigitalOutput(
            &mut self,
            pin: u8,
            level: bool,
        ) -> Result<DeviceDigitalOutputState, EdgeProxyError> {
            Ok(DeviceDigitalOutputState { pin, level })
        }

        async fn getDigitalOutput(
            &mut self,
            pin: u8,
        ) -> Result<DeviceDigitalOutputState, EdgeProxyError> {
            Ok(DeviceDigitalOutputState { pin, level: false })
        }

        async fn watchDigitalOutput(
            &mut self,
            _pin: u8,
        ) -> Result<EdgeStateStream, EdgeProxyError> {
            Err(EdgeProxyError {
                code: "UNSUPPORTED".to_string(),
                message: "watch is unused in firmware unit tests".to_string(),
            })
        }
    }

    #[tokio::test]
    async fn publishesExpressionToSharedStatus() {
        let status = Arc::new(FirmwareStatus::new("neutral"));
        let mut app = Esp32App::new(
            TestEdgeClient {
                expression: "neutral".to_string(),
            },
            Arc::clone(&status),
        );
        let state = app
            .setExpression("online")
            .await
            .expect("online expression must commit");
        assert_eq!(state.expression, "online");
        assert_eq!(status.snapshot().expression, "online");
    }
}
