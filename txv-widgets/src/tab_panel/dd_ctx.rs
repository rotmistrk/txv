//! Context struct for dropdown draw parameters.

use txv_core::prelude::*;

/// Bundled drawing params for dropdown rows.
pub(super) struct DdCtx {
    pub(super) box_w: u16,
    pub(super) inner_w: u16,
    pub(super) buf_h: u16,
    pub(super) border_style: Style,
}
