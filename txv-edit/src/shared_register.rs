//! Shared yank register — allows yank in one editor to paste in another.

use std::sync::{Arc, Mutex};

/// Shared yank register state.
#[derive(Default)]
pub struct SharedRegister {
    pub(crate) text: String,
    pub(crate) linewise: bool,
    pub(crate) block: bool,
}

impl SharedRegister {
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn linewise(&self) -> bool {
        self.linewise
    }
    pub fn block(&self) -> bool {
        self.block
    }
}

/// Thread-safe handle to a shared register.
pub type RegisterHandle = Arc<Mutex<SharedRegister>>;

/// Create a new shared register handle.
pub fn new_register() -> RegisterHandle {
    Arc::default()
}
