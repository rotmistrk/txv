//! TabEntry — data for a single tab dropdown entry.

use txv_core::prelude::Style;

pub(crate) struct TabEntry {
    pub(crate) label: String,
    pub(crate) dirty: bool,
    pub(crate) badge: Option<String>,
    pub(crate) badge_style: Option<Style>,
}
