//! Per-segment separator context for multi-tab draw.

use txv_core::prelude::*;

/// Per-segment separator context.
pub(super) struct SegCtx {
    pub(super) seg_idx: usize,
    pub(super) hidden_left: usize,
    pub(super) is_active: bool,
    pub(super) prev_active: bool,
    pub(super) cur_bg: Color,
    pub(super) prev_bg: Color,
}
