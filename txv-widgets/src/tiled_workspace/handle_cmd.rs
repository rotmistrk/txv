//! Command event handler for TiledWorkspace.

use std::any::Any;

use txv_core::event::CommandId;

use super::commands::*;
use super::types::SplitDir;
use super::TiledWorkspace;

impl TiledWorkspace {
    /// Handle a workspace command event. Returns true if consumed.
    pub fn handle_command(&mut self, id: CommandId, data: &Option<Box<dyn Any + Send>>) -> bool {
        match id {
            CM_TOGGLE_PANEL => {
                if let Some(panel_id) = extract_usize(data) {
                    self.toggle_panel(panel_id);
                }
                true
            }
            CM_SHOW_PANEL => {
                if let Some(panel_id) = extract_usize(data) {
                    self.show_panel(panel_id);
                }
                true
            }
            CM_HIDE_PANEL => {
                if let Some(panel_id) = extract_usize(data) {
                    self.hide_panel(panel_id);
                }
                true
            }
            CM_ZOOM => {
                self.toggle_zoom();
                true
            }
            CM_ZOOM_PANEL => {
                if let Some(panel_id) = extract_usize(data) {
                    self.zoom_panel(panel_id);
                }
                true
            }
            CM_UNZOOM => {
                self.unzoom();
                true
            }
            CM_FOCUS_PANEL => {
                if let Some(panel_id) = extract_usize(data) {
                    self.focus_panel(panel_id);
                }
                true
            }
            CM_FOCUS_DIRECTION => {
                if let Some(&(dx, dy)) = data.as_ref().and_then(|d| d.downcast_ref::<(i16, i16)>()) {
                    self.focus_direction(dx, dy);
                }
                true
            }
            CM_RESIZE_PANEL => {
                if let Some(&(dir, delta)) = data.as_ref().and_then(|d| d.downcast_ref::<(SplitDir, i16)>()) {
                    self.resize_panel(dir, delta);
                }
                true
            }
            CM_ACTIVATE_TAB => {
                if let Some(idx) = extract_usize(data) {
                    if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                        if idx < panel.tab_count() {
                            panel.set_active(idx);
                        }
                    }
                }
                true
            }
            CM_TAB_DROPDOWN => {
                if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                    panel.open_dropdown();
                }
                true
            }
            _ => false,
        }
    }

    /// Show a panel (unhide).
    pub fn show_panel(&mut self, id: usize) {
        if id < self.hidden.len() {
            self.hidden[id] = false;
            self.recompute_layout();
        }
    }

    /// Hide a panel.
    pub fn hide_panel(&mut self, id: usize) {
        if id < self.configs.len() && self.configs[id].hideable {
            self.hidden[id] = true;
            if self.group.focused_index() == id {
                self.focus_next_visible();
            }
            self.recompute_layout();
        }
    }

    /// Zoom a specific panel.
    pub fn zoom_panel(&mut self, id: usize) {
        self.zoomed = Some(id);
        self.recompute_layout();
    }

    /// Exit zoom.
    pub fn unzoom(&mut self) {
        self.zoomed = None;
        self.recompute_layout();
    }
}

fn extract_usize(data: &Option<Box<dyn Any + Send>>) -> Option<usize> {
    data.as_ref().and_then(|d| d.downcast_ref::<usize>()).copied()
}
