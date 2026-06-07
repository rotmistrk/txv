//! Badge rendering context for multi-tab overflow indicators.

use txv_core::prelude::*;

/// Badge rendering context for overflow indicators.
pub(super) struct BadgeRenderCtx<'a> {
    pub(super) tr: &'a str,
    pub(super) tr_len: u16,
    pub(super) badge: &'a str,
    pub(super) badge_len: u16,
    pub(super) prev_bg: Color,
    pub(super) badge_bg: Color,
    pub(super) badge_fg: Color,
}
