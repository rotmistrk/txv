//! Layout hints for a StatusBar child.

use super::gravity::Gravity;

/// Private storage for layout hints alongside each child.
pub(super) struct Hints {
    pub(super) priority: u8,
    pub(super) min_width: u16,
    pub(super) max_width: u16,
    pub(super) stretch: u16,
    pub(super) gravity: Gravity,
    /// Width captured at insertion time — used as fallback min when min_width is 0.
    pub(super) natural_width: u16,
    /// Width assigned by the last layout pass (to detect stretch inflation).
    pub(super) last_alloc: u16,
}
