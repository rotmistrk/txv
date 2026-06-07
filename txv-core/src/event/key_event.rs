//! A key event.

use super::{KeyCode, KeyMod};

/// A key event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KeyEvent {
    pub(crate) code: KeyCode,
    pub(crate) modifiers: KeyMod,
}

impl KeyEvent {
    pub const fn new(code: KeyCode, modifiers: KeyMod) -> Self {
        Self { code, modifiers }
    }

    pub fn code(&self) -> KeyCode {
        self.code
    }

    pub fn modifiers(&self) -> KeyMod {
        self.modifiers
    }
}
