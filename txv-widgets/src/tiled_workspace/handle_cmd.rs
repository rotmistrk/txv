//! Command event handler for TiledWorkspace.

use std::any::Any;

use txv_core::event::CommandId;

use super::commands::*;
use super::types::{PanelPosition, SplitDir};
use super::TiledWorkspace;

impl TiledWorkspace {
    /// Handle a workspace command event. Returns true if consumed.
    pub fn handle_command(&mut self, id: CommandId, data: &Option<Box<dyn Any + Send>>) -> bool {
        self.handle_panel_cmd(id, data)
            || self.handle_focus_resize_cmd(id)
            || self.handle_tab_cmd(id, data)
            || self.handle_subpanel_cmd(id)
    }

    fn handle_panel_cmd(&mut self, id: CommandId, data: &Option<Box<dyn Any + Send>>) -> bool {
        match id {
            CM_TW_TOGGLE_TREE => {
                if let Some(panel_id) = self.find_panel_by_position(PanelPosition::Left) {
                    self.toggle_panel(panel_id);
                }
                true
            }
            CM_TW_TOGGLE_TOOLS => {
                let pid = self
                    .find_panel_by_position(PanelPosition::Right)
                    .or_else(|| self.find_panel_by_position(PanelPosition::Bottom));
                if let Some(panel_id) = pid {
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
            _ => false,
        }
    }

    fn handle_focus_resize_cmd(&mut self, id: CommandId) -> bool {
        match id {
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
            _ => false,
        }
    }

    fn handle_tab_cmd(&mut self, id: CommandId, data: &Option<Box<dyn Any + Send>>) -> bool {
        match id {
            CM_TW_TAB_DROPDOWN => {
                if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                    panel.open_dropdown();
                }
                true
            }
            CM_TW_TAB_DROPDOWN_UP => {
                self.handle_dropdown_up();
                true
            }
            CM_TW_TAB_DROPDOWN_DOWN => {
                self.handle_dropdown_down();
                true
            }
            CM_TW_TAB_DROPDOWN_CLOSE => {
                if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                    panel.close_dropdown();
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
            CM_TW_TAB_NEXT | CM_TW_TAB_PREV => {
                self.handle_tab_nav(id);
                true
            }
            _ => false,
        }
    }

    fn handle_tab_nav(&mut self, id: CommandId) {
        let Some(panel) = self.panel_mut(self.group.focused_index()) else {
            return;
        };
        if id == CM_TW_TAB_NEXT {
            panel.tab_next();
        } else {
            panel.tab_prev();
        }
    }

    fn handle_subpanel_cmd(&mut self, id: CommandId) -> bool {
        match id {
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
            CM_TW_CLOSE_SUBPANEL => {
                self.collapse_subpanel();
                true
            }
            CM_TW_CLOSE_OTHER_SUBPANEL => {
                self.collapse_other_subpanel();
                true
            }
            CM_TW_EQUALIZE_SUBPANEL => {
                self.with_split_panel(|sp| sp.equalize());
                true
            }
            _ => false,
        }
    }

    fn handle_dropdown_up(&mut self) {
        let Some(panel) = self.panel_mut(self.group.focused_index()) else {
            return;
        };
        if panel.bar().dropdown_open() {
            panel.bar_mut().dropdown_move_up();
        } else if panel.tab_count() > 1 {
            panel.open_dropdown();
        }
    }

    fn handle_dropdown_down(&mut self) {
        let Some(panel) = self.panel_mut(self.group.focused_index()) else {
            return;
        };
        if panel.bar().dropdown_open() {
            panel.bar_mut().dropdown_move_down();
        } else if panel.tab_count() > 1 {
            panel.open_dropdown();
        }
    }

    /// Show a panel (unhide).
    pub fn show_panel(&mut self, id: usize) {
        if id < self.hidden.len() {
            self.hidden[id] = false;
            self.group.set_child_visible(id, true);
            self.recompute_layout();
        }
    }

    /// Hide a panel.
    pub fn hide_panel(&mut self, id: usize) {
        if id < self.configs.len() && self.configs[id].hideable {
            self.hidden[id] = true;
            self.group.set_child_visible(id, false);
            if self.group.focused_index() == id {
                self.focus_next_visible();
            }
            self.recompute_layout();
        }
    }

    /// Zoom a specific panel.
    pub fn zoom_panel(&mut self, id: usize) {
        self.zoomed = Some(id);
        self.sync_visibility();
        self.recompute_layout();
    }

    /// Exit zoom.
    pub fn unzoom(&mut self) {
        self.zoomed = None;
        self.sync_visibility();
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
