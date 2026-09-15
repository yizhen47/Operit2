#![allow(non_snake_case)]

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use operit_board_esp32::{Esp32ScreenMirror, Esp32ScreenMirrorRect};
use operit_node_edge::{
    EdgeScreenInputRequest, EdgeScreenInputState, EdgeScreenSnapshot, EdgeServiceError,
    ScreenService,
};

/// Board-backed generic display service for the authenticated EdgeLink.
pub struct Esp32ScreenService {
    mirror: Arc<Esp32ScreenMirror>,
    inputs: Arc<Mutex<VecDeque<EdgeScreenInputRequest>>>,
}

impl Esp32ScreenService {
    pub fn new(mirror: Arc<Esp32ScreenMirror>) -> Self {
        Self {
            mirror,
            inputs: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Takes all remote input events for execution by the firmware main loop.
    pub fn drainInputs(&self) -> Vec<EdgeScreenInputRequest> {
        let Ok(mut inputs) = self.inputs.lock() else {
            return Vec::new();
        };
        inputs.drain(..).collect()
    }
}

impl ScreenService for Esp32ScreenService {
    fn getScreenSnapshot(&self) -> Result<EdgeScreenSnapshot, EdgeServiceError> {
        let (width, height) = self.mirror.dimensions();
        let rowBytes = usize::from(width) * 2;
        let mut pixels = vec![0u8; rowBytes * usize::from(height)];
        self.mirror.withState(|background, rects| {
            for y in 0..usize::from(height) {
                for x in 0..usize::from(width) {
                    let pixel = mirroredPixel(background, rects, x as u16, y as u16);
                    let offset = y * rowBytes + x * 2;
                    pixels[offset] = (pixel >> 8) as u8;
                    pixels[offset + 1] = pixel as u8;
                }
            }
        });
        Ok(EdgeScreenSnapshot {
            width,
            height,
            format: "rgb565-be".to_string(),
            pixels,
        })
    }

    fn sendScreenInput(
        &self,
        request: EdgeScreenInputRequest,
    ) -> Result<EdgeScreenInputState, EdgeServiceError> {
        let action = request.action.trim().to_ascii_lowercase();
        if !matches!(action.as_str(), "tap" | "down" | "up" | "swipe") {
            return Err(EdgeServiceError::new(format!(
                "unsupported screen input action: {action}"
            )));
        }
        let (width, height) = self.mirror.dimensions();
        if request.x >= width || request.y >= height {
            return Err(EdgeServiceError::new("screen input point is outside display"));
        }
        if action == "swipe" && (request.endX.is_none() || request.endY.is_none()) {
            return Err(EdgeServiceError::new(
                "screen swipe requires endX and endY",
            ));
        }
        self.inputs
            .lock()
            .map_err(|error| EdgeServiceError::new(error.to_string()))?
            .push_back(EdgeScreenInputRequest {
                action: action.clone(),
                ..request
            });
        Ok(EdgeScreenInputState {
            accepted: true,
            action,
        })
    }
}

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
