//! Well-known command identifiers.
//!
//! Command ID ranges:
//! - `1..CM_CORE_MAX` — txv-core (quit, close, focus, etc.)
//! - `CM_CORE_MAX+1..CM_TXV_MAX` — txv-widgets (workspace, dialogs, etc.)
//! - `CM_TXV_MAX+1..` — application-specific commands

use crate::event::CommandId;

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

/// End of txv-core command range. Widgets use IDs above this.
pub const CM_CORE_MAX: CommandId = 99;

/// End of all TXV command ranges. Applications use IDs above this.
pub const CM_TXV_MAX: CommandId = 999;
