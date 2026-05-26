//! Layout hints for a StatusBar child.

use super::gravity::Gravity;

/// Private storage for layout hints alongside each child.
pub(super) struct Hints {
    pub(super) priority: u8,
    pub(super) min_width: u16,
    pub(super) max_width: u16,
    pub(super) stretch: u16,
    pub(super) gravity: Gravity,
}
