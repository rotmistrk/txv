//! Cached parse state snapshot for syntax highlighting.

use syntect::parsing::{ParseState, ScopeStack};

/// Cached parse state at a specific line boundary.
#[derive(Clone)]
pub(super) struct Snapshot {
    pub(super) parse: ParseState,
    pub(super) scope: ScopeStack,
}
