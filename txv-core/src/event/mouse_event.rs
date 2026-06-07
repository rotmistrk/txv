//! A mouse event.

use super::{KeyMod, MouseAction};

/// A mouse event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MouseEvent {
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) action: MouseAction,
    pub(crate) modifiers: KeyMod,
}

impl MouseEvent {
    pub fn new(x: u16, y: u16, action: MouseAction, modifiers: KeyMod) -> Self {
        Self {
            x,
            y,
            action,
            modifiers,
        }
    }

    pub fn x(&self) -> u16 {
        self.x
    }

    pub fn y(&self) -> u16 {
        self.y
    }

    pub fn action(&self) -> MouseAction {
        self.action
    }

    pub fn modifiers(&self) -> KeyMod {
        self.modifiers
    }
}
