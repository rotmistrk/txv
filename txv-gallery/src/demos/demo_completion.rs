//! Demo completion item.

use txv_core::prelude::*;

pub(crate) struct DemoCompletion {
    pub(crate) text: String,
}

impl Completion for DemoCompletion {
    fn text(&self) -> &str {
        &self.text
    }
    fn display(&self) -> &str {
        &self.text
    }
    fn kind(&self) -> &str {
        "widget"
    }
}
