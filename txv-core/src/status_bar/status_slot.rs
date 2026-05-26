//! StatusSlot — builder for adding a child View to the StatusBar.

use crate::view::View;

use super::gravity::Gravity;

/// Builder for adding a child to the StatusBar with layout hints.
/// All fields are private; configured via chained methods.
pub struct StatusSlot {
    view: Box<dyn View>,
    priority: u8,
    min_width: u16,
    max_width: u16,
    stretch: u16,
    gravity: Gravity,
}

impl StatusSlot {
    pub fn new(view: Box<dyn View>) -> Self {
        Self {
            view,
            priority: 5,
            min_width: 0,
            max_width: 0,
            stretch: 0,
            gravity: Gravity::Left,
        }
    }

    pub fn priority(mut self, p: u8) -> Self {
        self.priority = p;
        self
    }

    pub fn min_width(mut self, w: u16) -> Self {
        self.min_width = w;
        self
    }

    pub fn max_width(mut self, w: u16) -> Self {
        self.max_width = w;
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

    // --- Accessors for StatusBar to consume (crate-private) ---

    pub(super) fn take_view(self) -> (Box<dyn View>, u8, u16, u16, u16, Gravity) {
        (
            self.view,
            self.priority,
            self.min_width,
            self.max_width,
            self.stretch,
            self.gravity,
        )
    }
}
