#![allow(non_snake_case)]

/// Vertical distance, in logical pixels, that opens or closes the plugin shelf.
pub const SWIPE_THRESHOLD_PX: i16 = 40;

/// Surfaces owned by the firmware home shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HomeSurface {
    /// Robot face. This is the device home screen.
    Face,
    /// Phone-style minus-one screen. Plugin pages occupy this shelf later.
    PluginShelf,
}

/// Gestures the home shell understands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiGesture {
    SwipeDown,
    SwipeUp,
}

/// Firmware home-shell state machine. It does not run plugins.
pub struct HomeUi {
    surface: HomeSurface,
}

impl HomeUi {
    /// Starts on the robot-face home screen.
    pub fn new() -> Self {
        Self {
            surface: HomeSurface::Face,
        }
    }

    /// Returns the surface currently owned by the shell.
    pub fn surface(&self) -> HomeSurface {
        self.surface
    }

    /// Applies one gesture. Returns whether the visible surface changed.
    pub fn apply(&mut self, gesture: UiGesture) -> bool {
        let next = match (self.surface, gesture) {
            (HomeSurface::Face, UiGesture::SwipeDown) => HomeSurface::PluginShelf,
            (HomeSurface::PluginShelf, UiGesture::SwipeUp) => HomeSurface::Face,
            (surface, _) => surface,
        };
        if next == self.surface {
            return false;
        }
        self.surface = next;
        true
    }
}

/// Tracks one press-drag-release and emits a vertical swipe on lift.
pub struct SwipeTracker {
    origin: Option<(i16, i16)>,
    last: Option<(i16, i16)>,
}

impl SwipeTracker {
    /// Creates an idle tracker.
    pub fn new() -> Self {
        Self {
            origin: None,
            last: None,
        }
    }

    /// Feeds one sample. A swipe is reported when the finger lifts.
    pub fn onSample(&mut self, point: Option<(u16, u16)>) -> Option<UiGesture> {
        match point {
            Some((x, y)) => {
                let sample = (x as i16, y as i16);
                if self.origin.is_none() {
                    self.origin = Some(sample);
                }
                self.last = Some(sample);
                None
            }
            None => match (self.origin.take(), self.last.take()) {
                (Some(origin), Some(last)) => {
                    gestureFromSwipe(origin.0, origin.1, last.0, last.1)
                }
                _ => None,
            },
        }
    }
}

/// Classifies a drag as a vertical home-shell swipe.
pub fn gestureFromSwipe(startX: i16, startY: i16, endX: i16, endY: i16) -> Option<UiGesture> {
    let dx = endX.saturating_sub(startX);
    let dy = endY.saturating_sub(startY);
    if dy.abs() <= SWIPE_THRESHOLD_PX || dy.abs() <= dx.abs() {
        return None;
    }
    if dy > 0 {
        Some(UiGesture::SwipeDown)
    } else {
        Some(UiGesture::SwipeUp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swipeDownOpensPluginShelf() {
        let mut home = HomeUi::new();
        assert_eq!(home.surface(), HomeSurface::Face);
        assert!(home.apply(UiGesture::SwipeDown));
        assert_eq!(home.surface(), HomeSurface::PluginShelf);
        assert!(!home.apply(UiGesture::SwipeDown));
    }

    #[test]
    fn swipeUpReturnsToFace() {
        let mut home = HomeUi::new();
        home.apply(UiGesture::SwipeDown);
        assert!(home.apply(UiGesture::SwipeUp));
        assert_eq!(home.surface(), HomeSurface::Face);
        assert!(!home.apply(UiGesture::SwipeUp));
    }

    #[test]
    fn verticalDragMapsToSwipe() {
        assert_eq!(
            gestureFromSwipe(120, 20, 118, 90),
            Some(UiGesture::SwipeDown)
        );
        assert_eq!(
            gestureFromSwipe(120, 200, 122, 40),
            Some(UiGesture::SwipeUp)
        );
        assert_eq!(gestureFromSwipe(10, 10, 80, 12), None);
        assert_eq!(gestureFromSwipe(120, 20, 120, 50), None);
    }

    #[test]
    fn trackerEmitsSwipeOnRelease() {
        let mut tracker = SwipeTracker::new();
        assert_eq!(tracker.onSample(Some((120, 20))), None);
        assert_eq!(tracker.onSample(Some((118, 90))), None);
        assert_eq!(tracker.onSample(None), Some(UiGesture::SwipeDown));
        assert_eq!(tracker.onSample(None), None);
    }
}
