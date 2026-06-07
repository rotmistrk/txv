//! Shared resize helpers for status bar items.

use txv_core::prelude::*;

/// Resize a ViewState's width if it differs from the given value.
pub fn resize_width_to(state: &mut ViewState, w: u16) {
    let b = state.bounds();
    if b.w() != w {
        state.set_bounds(Rect::new(b.x(), b.y(), w, b.h()));
    }
}
