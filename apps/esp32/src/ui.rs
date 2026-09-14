#![allow(non_snake_case)]

use std::sync::atomic::{AtomicU8, Ordering};

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

/// Gestures the home shell understands. Touch Host can feed these later.
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

/// One-slot mailbox so the HTTP home page can post a swipe without extra tasks.
pub struct GestureMailbox {
    pending: AtomicU8,
}

impl GestureMailbox {
    const NONE: u8 = 0;
    const DOWN: u8 = 1;
    const UP: u8 = 2;

    /// Creates an empty mailbox.
    pub fn new() -> Self {
        Self {
            pending: AtomicU8::new(Self::NONE),
        }
    }

    /// Replaces any unread gesture with `gesture`.
    pub fn post(&self, gesture: UiGesture) {
        let code = match gesture {
            UiGesture::SwipeDown => Self::DOWN,
            UiGesture::SwipeUp => Self::UP,
        };
        self.pending.store(code, Ordering::SeqCst);
    }

    /// Takes the pending gesture, if any.
    pub fn take(&self) -> Option<UiGesture> {
        match self.pending.swap(Self::NONE, Ordering::SeqCst) {
            Self::DOWN => Some(UiGesture::SwipeDown),
            Self::UP => Some(UiGesture::SwipeUp),
            _ => None,
        }
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
    fn mailboxStoresLatestGesture() {
        let mailbox = GestureMailbox::new();
        mailbox.post(UiGesture::SwipeDown);
        mailbox.post(UiGesture::SwipeUp);
        assert_eq!(mailbox.take(), Some(UiGesture::SwipeUp));
        assert_eq!(mailbox.take(), None);
    }
}
