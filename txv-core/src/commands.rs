//! Well-known command identifiers.
//!
//! Command ID ranges:
//! - `1..CM_CORE_MAX` — txv-core (quit, close, focus, etc.)
//! - `CM_CORE_MAX+1..CM_TXV_MAX` — txv-widgets (workspace, dialogs, etc.)
//! - `CM_TXV_MAX+1..` — application-specific commands

use crate::event::CommandId;
use crate::view::ViewId;

pub const CM_QUIT: CommandId = 1;
pub const CM_CLOSE: CommandId = 2;
pub const CM_FOCUS_NEXT: CommandId = 3;
pub const CM_FOCUS_PREV: CommandId = 4;
pub const CM_HELP: CommandId = 5;
pub const CM_MENU: CommandId = 6;
pub const CM_OK: CommandId = 7;
pub const CM_CANCEL: CommandId = 8;
pub const CM_TICK: CommandId = 9;
/// Force full screen repaint (invalidate backend + new buffer).
pub const CM_REPAINT: CommandId = 10;

/// A child requests repositioning. Data: `Box<RepositionRequest>`.
/// Handled by any GroupState in postprocess: finds child by view_id, sets origin + size.
pub const CM_REPOSITION: CommandId = 11;

/// End of txv-core command range. Widgets use IDs above this.
pub const CM_CORE_MAX: CommandId = 99;

/// Data payload for CM_REPOSITION.
pub struct RepositionRequest {
    pub(crate) view_id: ViewId,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) offset_x: i16,
    pub(crate) offset_y: i16,
    pub(crate) relative_to: Option<ViewId>,
}

impl RepositionRequest {
    pub fn new(view_id: ViewId, width: u16, height: u16) -> Self {
        Self {
            view_id,
            width,
            height,
            offset_x: 0,
            offset_y: 0,
            relative_to: None,
        }
    }

    pub fn with_offset(mut self, x: i16, y: i16) -> Self {
        self.offset_x = x;
        self.offset_y = y;
        self
    }

    pub fn relative_to(mut self, view_id: ViewId) -> Self {
        self.relative_to = Some(view_id);
        self
    }

    pub fn view_id(&self) -> ViewId {
        self.view_id
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn offset_x(&self) -> i16 {
        self.offset_x
    }

    pub fn offset_y(&self) -> i16 {
        self.offset_y
    }

    pub fn relative_to_view(&self) -> Option<ViewId> {
        self.relative_to
    }
}

/// End of all TXV command ranges. Applications use IDs above this.
pub const CM_TXV_MAX: CommandId = 999;
