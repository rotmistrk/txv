//! Shared context for multi-tab segment drawing.

use txv_core::prelude::*;

/// Shared context for multi-tab segment drawing.
pub(super) struct MultiDrawCtx<'a> {
    pub(super) order: &'a [usize],
    pub(super) vis_start: usize,
    pub(super) fill_bg: Color,
}
