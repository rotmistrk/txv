//! StatusSlot — builder for adding a child View to the StatusBar.

use crate::view::View;

use super::gravity::Gravity;

/// Builder for adding a child to the StatusBar with layout hints.
pub struct StatusSlot {
    view: Box<dyn View>,
    priority: u8,
    stretch: u16,
    gravity: Gravity,
}

impl StatusSlot {
    pub fn new(view: Box<dyn View>) -> Self {
        Self {
            view,
            priority: 5,
            stretch: 0,
            gravity: Gravity::Left,
        }
    }

    pub fn priority(mut self, p: u8) -> Self {
        self.priority = p;
        self
    }

    /// Kept for API compatibility (ignored).
    pub fn min_width(self, _w: u16) -> Self {
        self
    }

    /// Kept for API compatibility (ignored).
    pub fn max_width(self, _w: u16) -> Self {
        self
    }

    pub fn stretch(mut self, s: u16) -> Self {
        self.stretch = s;
        self
    }

    pub fn gravity(mut self, g: Gravity) -> Self {
        self.gravity = g;
        self
    }

    pub(super) fn take_view(self) -> (Box<dyn View>, u8, u16, Gravity) {
        (self.view, self.priority, self.stretch, self.gravity)
    }
}
