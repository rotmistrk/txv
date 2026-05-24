//! Command IDs for TiledWorkspace.
//!
//! All commands use the `CM_TW_` prefix. Commands that act on the focused
//! panel or have a fixed semantic target require no payload.
//! A few programmatic commands still accept a payload (marked below).

use txv_core::commands::CM_CORE_MAX;
use txv_core::event::CommandId;

/// Base for workspace commands.
pub const CM_TW_BASE: CommandId = CM_CORE_MAX + 1;

// --- Panel visibility ---

/// Toggle tree (left) panel. No payload.
pub const CM_TW_TOGGLE_TREE: CommandId = CM_TW_BASE;
/// Toggle tools (right/bottom) panel. No payload.
pub const CM_TW_TOGGLE_TOOLS: CommandId = CM_TW_BASE + 1;
/// Show a panel. Payload: `usize` (panel ID).
pub const CM_TW_SHOW_PANEL: CommandId = CM_TW_BASE + 2;
/// Hide a panel. Payload: `usize` (panel ID).
pub const CM_TW_HIDE_PANEL: CommandId = CM_TW_BASE + 3;
/// Focus a panel by ID. Payload: `usize` (panel ID).
pub const CM_TW_FOCUS_PANEL: CommandId = CM_TW_BASE + 20;

// --- Zoom ---

/// Toggle zoom on focused panel. No payload.
pub const CM_TW_ZOOM: CommandId = CM_TW_BASE + 4;

// --- Focus navigation ---

/// Focus panel to the left. No payload.
pub const CM_TW_FOCUS_LEFT: CommandId = CM_TW_BASE + 5;
/// Focus panel to the right. No payload.
pub const CM_TW_FOCUS_RIGHT: CommandId = CM_TW_BASE + 6;
/// Focus panel above. No payload.
pub const CM_TW_FOCUS_UP: CommandId = CM_TW_BASE + 7;
/// Focus panel below. No payload.
pub const CM_TW_FOCUS_DOWN: CommandId = CM_TW_BASE + 8;

// --- Panel resize ---

/// Grow focused panel horizontally. No payload.
pub const CM_TW_GROW_H: CommandId = CM_TW_BASE + 9;
/// Shrink focused panel horizontally. No payload.
pub const CM_TW_SHRINK_H: CommandId = CM_TW_BASE + 10;
/// Grow focused panel vertically. No payload.
pub const CM_TW_GROW_V: CommandId = CM_TW_BASE + 11;
/// Shrink focused panel vertically. No payload.
pub const CM_TW_SHRINK_V: CommandId = CM_TW_BASE + 12;

// --- Tabs ---

/// Open tab dropdown on focused panel. No payload.
pub const CM_TW_TAB_DROPDOWN: CommandId = CM_TW_BASE + 13;
/// Move dropdown selection up. No payload.
pub const CM_TW_TAB_DROPDOWN_UP: CommandId = CM_TW_BASE + 21;
/// Move dropdown selection down. No payload.
pub const CM_TW_TAB_DROPDOWN_DOWN: CommandId = CM_TW_BASE + 22;
/// Close dropdown / confirm selection. No payload.
pub const CM_TW_TAB_DROPDOWN_CLOSE: CommandId = CM_TW_BASE + 23;
/// Activate tab by index in focused panel. Payload: `usize`.
pub const CM_TW_ACTIVATE_TAB: CommandId = CM_TW_BASE + 14;

// --- Layout ---

/// Cycle layout mode. No payload.
pub const CM_TW_LAYOUT_CYCLE: CommandId = CM_TW_BASE + 15;

// --- Subpanel ---

/// Cycle focus between subpanels. No payload.
pub const CM_TW_CYCLE_SUBPANEL: CommandId = CM_TW_BASE + 16;
/// Move tab to next subpanel. No payload.
pub const CM_TW_MOVE_TAB_SUBPANEL: CommandId = CM_TW_BASE + 17;
/// Grow focused subpanel. No payload.
pub const CM_TW_GROW_SUBPANEL: CommandId = CM_TW_BASE + 18;
/// Shrink focused subpanel. No payload.
pub const CM_TW_SHRINK_SUBPANEL: CommandId = CM_TW_BASE + 19;
/// Split focused panel horizontally (top/bottom). No payload.
pub const CM_TW_SPLIT_H: CommandId = CM_TW_BASE + 27;
/// Split focused panel vertically (left/right). No payload.
pub const CM_TW_SPLIT_V: CommandId = CM_TW_BASE + 28;
/// Close focused subpanel. No payload.
pub const CM_TW_CLOSE_SUBPANEL: CommandId = CM_TW_BASE + 29;
/// Close other subpanel (keep focused). No payload.
pub const CM_TW_CLOSE_OTHER_SUBPANEL: CommandId = CM_TW_BASE + 30;
/// Equalize subpanel proportions. No payload.
pub const CM_TW_EQUALIZE_SUBPANEL: CommandId = CM_TW_BASE + 31;

/// End of workspace command range.
pub const CM_TW_MAX: CommandId = CM_TW_BASE + 49;

// --- Tab cycling ---

/// Next tab in focused panel. No payload.
pub const CM_TW_TAB_NEXT: CommandId = CM_TW_BASE + 24;
/// Previous tab in focused panel. No payload.
pub const CM_TW_TAB_PREV: CommandId = CM_TW_BASE + 25;
/// Close active tab in focused panel. No payload.
pub const CM_TW_TAB_CLOSE: CommandId = CM_TW_BASE + 26;
