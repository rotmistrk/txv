//! Command IDs for TiledWorkspace.
//!
//! External integrations (scripting, MCP, configuration) interact with
//! TiledWorkspace by emitting these command events into the EventQueue.
//! Direct method calls are internal implementation — use commands for
//! decoupled interaction.
//!
//! Command payloads use `Box<dyn Any>`. Payload types:
//! - `PanelId` (usize) — for panel-targeted commands
//! - `(PanelId, usize)` — for (panel, tab_index) commands
//! - `(SplitDir, i16)` — for resize commands
//! - None — for commands that act on the focused panel

use txv_core::commands::CM_CORE_MAX;
use txv_core::event::CommandId;

/// Base for workspace commands.
pub const CM_WORKSPACE_BASE: CommandId = CM_CORE_MAX + 1;

/// Toggle panel visibility. Payload: `PanelId`.
pub const CM_TOGGLE_PANEL: CommandId = CM_WORKSPACE_BASE;
/// Show a hidden panel. Payload: `PanelId`.
pub const CM_SHOW_PANEL: CommandId = CM_WORKSPACE_BASE + 1;
/// Hide a panel. Payload: `PanelId`.
pub const CM_HIDE_PANEL: CommandId = CM_WORKSPACE_BASE + 2;
/// Toggle zoom on focused panel. No payload.
pub const CM_ZOOM: CommandId = CM_WORKSPACE_BASE + 3;
/// Zoom a specific panel. Payload: `PanelId`.
pub const CM_ZOOM_PANEL: CommandId = CM_WORKSPACE_BASE + 4;
/// Exit zoom. No payload.
pub const CM_UNZOOM: CommandId = CM_WORKSPACE_BASE + 5;
/// Focus a panel by ID. Payload: `PanelId`.
pub const CM_FOCUS_PANEL: CommandId = CM_WORKSPACE_BASE + 6;
/// Focus panel in direction. Payload: `(i16, i16)` as (dx, dy).
pub const CM_FOCUS_DIRECTION: CommandId = CM_WORKSPACE_BASE + 7;
/// Resize panel border. Payload: `(SplitDir, i16)`.
pub const CM_RESIZE_PANEL: CommandId = CM_WORKSPACE_BASE + 8;
/// Activate tab by index in focused panel. Payload: `usize` (tab index).
pub const CM_ACTIVATE_TAB: CommandId = CM_WORKSPACE_BASE + 9;
/// Close tab by index. Payload: `(PanelId, usize)`.
pub const CM_CLOSE_TAB: CommandId = CM_WORKSPACE_BASE + 10;
/// Move active tab to another panel. Payload: `PanelId` (target).
pub const CM_MOVE_TAB: CommandId = CM_WORKSPACE_BASE + 11;
/// Open tab dropdown on focused panel. No payload.
pub const CM_TAB_DROPDOWN: CommandId = CM_WORKSPACE_BASE + 12;
/// Split focused panel's subpanel area. No payload.
pub const CM_SPLIT_SUBPANEL: CommandId = CM_WORKSPACE_BASE + 13;
/// Move tab to next subpanel (creates split if needed). No payload.
pub const CM_MOVE_TAB_SUBPANEL: CommandId = CM_WORKSPACE_BASE + 14;
/// Merge subpanels back into one. No payload.
pub const CM_UNSPLIT: CommandId = CM_WORKSPACE_BASE + 15;
/// Cycle focus between subpanels. No payload.
pub const CM_CYCLE_SUBPANEL: CommandId = CM_WORKSPACE_BASE + 16;
/// Grow focused subpanel. No payload.
pub const CM_GROW_SUBPANEL: CommandId = CM_WORKSPACE_BASE + 17;
/// Shrink focused subpanel. No payload.
pub const CM_SHRINK_SUBPANEL: CommandId = CM_WORKSPACE_BASE + 18;

/// End of workspace command range.
pub const CM_WORKSPACE_MAX: CommandId = CM_WORKSPACE_BASE + 49;
