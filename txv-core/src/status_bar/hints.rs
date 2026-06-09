//! Layout hints for a StatusBar child.

use super::gravity::Gravity;

/// Private storage for layout hints alongside each child.
pub(super) struct Hints {
    pub(super) priority: u8,
    pub(super) stretch: u16,
    pub(super) gravity: Gravity,
    /// Width captured at insertion (child's natural size).
    pub(super) natural_width: u16,
}
