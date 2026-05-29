//! Field accessor methods for TiledWorkspace.

use txv_core::event::CommandId;
use txv_core::prelude::*;

use super::types::{LayoutMode, PanelId};
use super::TiledWorkspace;

impl TiledWorkspace {
    /// Whether the workspace is currently in wide layout mode.
    pub fn is_wide(&self) -> bool {
        self.is_wide
    }

    /// Whether a panel is zoomed.
    pub fn is_zoomed(&self) -> bool {
        self.zoomed.is_some()
    }

    /// Set the wide threshold (above this, layout is wide).
    pub fn set_wide_threshold(&mut self, threshold: u16) {
        self.wide_threshold = threshold;
    }

    /// Set the narrow threshold (below this, layout is narrow).
    pub fn set_narrow_threshold(&mut self, threshold: u16) {
        self.narrow_threshold = threshold;
    }

    /// Set the layout mode (Auto/Wide/Narrow).
    pub fn set_layout_mode(&mut self, mode: LayoutMode) {
        self.layout_mode = mode;
        self.recompute_layout();
    }

    /// Get the current layout mode.
    pub fn layout_mode(&self) -> LayoutMode {
        self.layout_mode
    }

    /// Enable/disable horizontal divider gaps.
    pub fn set_h_divider_gaps(&mut self, enabled: bool) {
        self.h_divider_gaps = enabled;
    }

    /// Enable/disable vertical divider gaps.
    pub fn set_v_divider_gaps(&mut self, enabled: bool) {
        self.v_divider_gaps = enabled;
    }

    /// Check if a panel is hidden.
    pub fn is_hidden(&self, id: PanelId) -> bool {
        self.hidden.get(id).copied().unwrap_or(false)
    }

    /// Hide a panel by ID (bypasses hideable check — for construction).
    pub fn set_hidden(&mut self, id: PanelId, hidden: bool) {
        if id < self.hidden.len() {
            self.hidden[id] = hidden;
            if hidden && self.group.focused_index() == id {
                self.focus_next_visible();
            }
            self.recompute_layout();
        }
    }

    /// Set zoom to follow a specific panel (or clear zoom).
    pub fn set_zoomed(&mut self, id: Option<PanelId>) {
        self.zoomed = id;
        self.recompute_layout();
    }

    /// Get the zoomed panel ID (if any).
    pub fn zoomed_panel(&self) -> Option<PanelId> {
        self.zoomed
    }

    /// Access a child view by index.
    pub fn child(&self, id: PanelId) -> Option<&dyn View> {
        self.group.child(id)
    }

    /// Access a child view mutably by index.
    pub fn child_mut(&mut self, id: PanelId) -> Option<&mut Box<dyn View>> {
        self.group.child_mut(id)
    }

    /// Dispatch an event to the focused child.
    pub fn dispatch(&mut self, event: &Event) -> HandleResult {
        self.group.dispatch(event)
    }

    /// Process a command through TiledWorkspace's command handler.
    /// Returns Consumed if TiledWorkspace handled it, Ignored otherwise.
    pub fn handle_command_event(
        &mut self,
        id: CommandId,
        data: &Option<Box<dyn std::any::Any + Send>>,
    ) -> HandleResult {
        if self.handle_command(id, data) {
            HandleResult::Consumed
        } else {
            HandleResult::Ignored
        }
    }

    /// Emit a command through the group's event sink.
    pub fn put_command(&mut self, id: CommandId, data: Option<Box<dyn std::any::Any + Send>>) {
        self.group.put_command(id, data);
    }

    /// Number of panels.
    pub fn panel_count(&self) -> usize {
        self.configs.len()
    }

    /// Insert an extra child (drawn on top of panels). Returns its index.
    pub fn insert_extra(&mut self, child: Box<dyn View>) -> usize {
        let idx = self.group.child_count();
        self.group.insert(child);
        self.group.mark_dirty();
        idx
    }

    /// Remove an extra child by index. Panics if index is within panel range.
    pub fn remove_extra(&mut self, idx: usize) -> Box<dyn View> {
        assert!(idx >= self.configs.len(), "cannot remove a panel child");
        self.group.mark_dirty();
        self.group.remove(idx)
    }
}
