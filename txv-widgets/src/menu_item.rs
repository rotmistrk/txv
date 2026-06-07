//! MenuItem — a single menu entry.

use txv_core::prelude::*;

pub struct MenuItem {
    pub(crate) label: String,
    pub(crate) command: CommandId,
    pub(crate) enabled: bool,
}

impl MenuItem {
    pub fn new(label: impl Into<String>, command: CommandId) -> Self {
        Self {
            label: label.into(),
            command,
            enabled: true,
        }
    }
}
