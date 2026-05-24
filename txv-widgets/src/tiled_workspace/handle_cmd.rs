//! Command event handler for TiledWorkspace.

use std::any::Any;

use txv_core::event::CommandId;

use super::commands::*;
use super::types::{PanelPosition, SplitDir};
use super::TiledWorkspace;

impl TiledWorkspace {
    /// Handle a workspace command event. Returns true if consumed.
    pub fn handle_command(&mut self, id: CommandId, data: &Option<Box<dyn Any + Send>>) -> bool {
        match id {
            CM_TW_TOGGLE_TREE => {
                if let Some(panel_id) = self.find_panel_by_position(PanelPosition::Left) {
                    self.toggle_panel(panel_id);
                }
                true
            }
            CM_TW_TOGGLE_TOOLS => {
                let id = self
                    .find_panel_by_position(PanelPosition::Right)
                    .or_else(|| self.find_panel_by_position(PanelPosition::Bottom));
                if let Some(panel_id) = id {
                    self.toggle_panel(panel_id);
                }
                true
            }
            CM_TW_SHOW_PANEL => {
                if let Some(panel_id) = extract_usize(data) {
                    self.show_panel(panel_id);
                }
                true
            }
            CM_TW_HIDE_PANEL => {
                if let Some(panel_id) = extract_usize(data) {
                    self.hide_panel(panel_id);
                }
                true
            }
            CM_TW_FOCUS_PANEL => {
                if let Some(panel_id) = extract_usize(data) {
                    self.focus_panel(panel_id);
                }
                true
            }
            CM_TW_ZOOM => {
                self.toggle_zoom();
                true
            }
            CM_TW_FOCUS_LEFT => {
                self.focus_direction(-1, 0);
                true
            }
            CM_TW_FOCUS_RIGHT => {
                self.focus_direction(1, 0);
                true
            }
            CM_TW_FOCUS_UP => {
                self.focus_direction(0, -1);
                true
            }
            CM_TW_FOCUS_DOWN => {
                self.focus_direction(0, 1);
                true
            }
            CM_TW_GROW_H => {
                self.resize_panel(SplitDir::Horizontal, 2);
                true
            }
            CM_TW_SHRINK_H => {
                self.resize_panel(SplitDir::Horizontal, -2);
                true
            }
            CM_TW_GROW_V => {
                self.resize_panel(SplitDir::Vertical, 1);
                true
            }
            CM_TW_SHRINK_V => {
                self.resize_panel(SplitDir::Vertical, -1);
                true
            }
            CM_TW_TAB_DROPDOWN => {
                if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                    panel.bar_mut().open_dropdown();
                }
                true
            }
            CM_TW_TAB_DROPDOWN_UP => {
                if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                    if panel.bar().dropdown_open() {
                        panel.bar_mut().dropdown_move_up();
                    } else if panel.tab_count() > 1 {
                        panel.bar_mut().open_dropdown();
                    }
                }
                true
            }
            CM_TW_TAB_DROPDOWN_DOWN => {
                if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                    if panel.bar().dropdown_open() {
                        panel.bar_mut().dropdown_move_down();
                    } else if panel.tab_count() > 1 {
                        panel.bar_mut().open_dropdown();
                    }
                }
                true
            }
            CM_TW_TAB_DROPDOWN_CLOSE => {
                if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                    panel.bar_mut().close_dropdown();
                }
                true
            }
            CM_TW_ACTIVATE_TAB => {
                if let Some(idx) = extract_usize(data) {
                    if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                        panel.activate_by_label(idx + 1);
                    }
                }
                true
            }
            CM_TW_LAYOUT_CYCLE => {
                self.cycle_layout();
                true
            }
            CM_TW_TAB_NEXT => {
                if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                    panel.tab_next();
                }
                true
            }
            CM_TW_TAB_PREV => {
                if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                    panel.tab_prev();
                }
                true
            }
            CM_TW_CYCLE_SUBPANEL => {
                self.with_split_panel(|sp| sp.cycle_focus());
                true
            }
            CM_TW_MOVE_TAB_SUBPANEL => {
                self.move_tab_to_subpanel();
                true
            }
            CM_TW_GROW_SUBPANEL => {
                self.with_split_panel(|sp| sp.grow_focused());
                true
            }
            CM_TW_SHRINK_SUBPANEL => {
                self.with_split_panel(|sp| sp.shrink_focused());
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
    data.as_ref().and_then(|d| {
        d.downcast_ref::<usize>()
            .copied()
            .or_else(|| d.downcast_ref::<u16>().map(|v| *v as usize))
    })
}
