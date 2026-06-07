//! HelpCompletion — test completion item.

use txv_core::prelude::*;

pub(crate) struct HelpCompletion;

impl Completion for HelpCompletion {
    fn text(&self) -> &str {
        "help"
    }
    fn display(&self) -> &str {
        "help"
    }
    fn kind(&self) -> &str {
        "cmd"
    }
}
