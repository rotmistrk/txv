//! View trait implementation for TiledWorkspace.

use txv_core::prelude::*;

use super::types::{PanelPosition, SplitDir};
use super::TiledWorkspace;

impl View for TiledWorkspace {
    delegate_group_state!(group, override { set_bounds, draw, handle });

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn set_bounds(&mut self, r: Rect) {
        self.group.set_bounds(r);
        self.group.mark_dirty();
        self.recompute_layout();
    }

    fn draw(&mut self) {
        let w = self.group.buffer_mut().width();
        let h = self.group.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        self.group.buffer_mut().fill(' ', Style::default());
        self.draw_chrome();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Command { id, data, .. } = event {
            if self.handle_command(*id, data) {
                return HandleResult::Consumed;
            }
        }
        if matches!(event, Event::Tick) {
            for i in 0..self.configs.len() {
                if let Some(child) = self.group.child_mut(i) {
                    child.handle(event);
                }
            }
            return HandleResult::Ignored;
        }
        let Event::Key(key) = event else {
            return self.group.dispatch(event);
        };
        if !self.handle_keys {
            return self.group.dispatch(event);
        }
        if let Some(result) = self.dispatch_key(key) {
            return result;
        }
        self.group.dispatch(event)
    }
}

impl TiledWorkspace {
    fn dispatch_key(&mut self, key: &KeyEvent) -> Option<HandleResult> {
        let km = self.keymap.clone();
        // Tab dropdown keys checked first — they may need to override focus keys
        if let Some(r) = self.dispatch_tab_keys(key, &km) {
            return Some(r);
        }
        if let Some(r) = self.dispatch_panel_keys(key, &km) {
            return Some(r);
        }
        if let Some(r) = self.dispatch_resize_keys(key, &km) {
            return Some(r);
        }
        None
    }

    fn dispatch_panel_keys(&mut self, key: &KeyEvent, km: &super::keymap::WorkspaceKeymap) -> Option<HandleResult> {
        if km.matches(key, &km.toggle_tree) {
            if let Some(id) = self.find_panel_by_position(PanelPosition::Left) {
                self.toggle_panel(id);
                return Some(HandleResult::Consumed);
            }
        }
        if km.matches(key, &km.toggle_tools) {
            let id = self
                .find_panel_by_position(PanelPosition::Right)
                .or_else(|| self.find_panel_by_position(PanelPosition::Bottom));
            if let Some(id) = id {
                self.toggle_panel(id);
                return Some(HandleResult::Consumed);
            }
        }
        if km.matches(key, &km.zoom) {
            self.toggle_zoom();
            return Some(HandleResult::Consumed);
        }
        // Skip focus navigation when panel has dropdown open
        let dd_open = self
            .panel(self.group.focused_index())
            .is_some_and(|p| p.dropdown_open());
        if !dd_open {
            if km.matches(key, &km.focus_left) {
                self.focus_direction(-1, 0);
                return Some(HandleResult::Consumed);
            }
            if km.matches(key, &km.focus_right) {
                self.focus_direction(1, 0);
                return Some(HandleResult::Consumed);
            }
            if km.matches(key, &km.focus_up) {
                self.focus_direction(0, -1);
                return Some(HandleResult::Consumed);
            }
            if km.matches(key, &km.focus_down) {
                self.focus_direction(0, 1);
                return Some(HandleResult::Consumed);
            }
        }
        None
    }

    fn dispatch_resize_keys(&mut self, key: &KeyEvent, km: &super::keymap::WorkspaceKeymap) -> Option<HandleResult> {
        if km.matches(key, &km.resize_left) {
            self.resize_panel(SplitDir::Horizontal, -1);
            return Some(HandleResult::Consumed);
        }
        if km.matches(key, &km.resize_right) {
            self.resize_panel(SplitDir::Horizontal, 1);
            return Some(HandleResult::Consumed);
        }
        if km.matches(key, &km.resize_up) {
            self.resize_panel(SplitDir::Vertical, -1);
            return Some(HandleResult::Consumed);
        }
        if km.matches(key, &km.resize_down) {
            self.resize_panel(SplitDir::Vertical, 1);
            return Some(HandleResult::Consumed);
        }
        None
    }

    fn dispatch_tab_keys(&mut self, key: &KeyEvent, km: &super::keymap::WorkspaceKeymap) -> Option<HandleResult> {
        if km.matches(key, &km.tab_dropdown) {
            if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                if panel.tab_count() > 1 {
                    panel.open_dropdown();
                    return Some(HandleResult::Consumed);
                }
            }
            return None;
        }
        if km.matches(key, &km.tab_dropdown_down) || km.matches(key, &km.tab_dropdown_up) {
            if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                if panel.tab_count() > 1 && !panel.dropdown_open() {
                    panel.open_dropdown();
                    return Some(HandleResult::Consumed);
                }
            }
            return None;
        }
        if km.matches(key, &km.layout_cycle) {
            self.cycle_layout();
            return Some(HandleResult::Consumed);
        }
        if km.matches(key, &km.tab_next) {
            if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                panel.tab_next();
            }
            return Some(HandleResult::Consumed);
        }
        if km.matches(key, &km.tab_prev) {
            if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                panel.tab_prev();
            }
            return Some(HandleResult::Consumed);
        }
        if km.matches(key, &km.subpanel_move_tab) {
            self.move_tab_to_subpanel();
            return Some(HandleResult::Consumed);
        }
        None
    }

    pub(crate) fn find_panel_by_position(&self, pos: PanelPosition) -> Option<usize> {
        self.configs.iter().position(|c| c.position == pos)
    }

    /// Focus the panel in the given direction relative to current.
    pub(crate) fn focus_direction(&mut self, dx: i16, dy: i16) {
        if self.zoomed.is_some() {
            self.focus_direction_zoomed(dx, dy);
            return;
        }
        self.focus_direction_normal(dx, dy);
    }
}
