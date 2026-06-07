//! StatusItem — a single key→command entry in a StatusBar.

use txv_core::prelude::*;

pub struct StatusItem {
    pub(crate) key: KeyEvent,
    pub(crate) command: CommandId,
    pub(crate) label: String,
}
