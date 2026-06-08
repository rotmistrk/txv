//! View trait implementation for TiledWorkspace.

use txv_core::prelude::*;

use super::types::{PanelPosition, SplitDir};
use super::TiledWorkspace;

impl View for TiledWorkspace {
    delegate_group_state!(group, override { set_bounds, draw, handle });

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
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

    fn render(&mut self) -> bool {
        let own_dirty = self.group.is_dirty();
        let mut child_drew = false;
        for i in 0..self.configs.len() {
            if !self.is_panel_visible(i) {
                continue;
            }
            if let Some(child) = self.group.child_mut(i) {
                child_drew |= child.render();
            }
        }
        if own_dirty {
            self.draw();
            self.blit_visible_children();
            self.group.mark_redrawn();
            return true;
        }
        if child_drew {
            self.blit_visible_children();
            return true;
        }
        false
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
    fn blit_visible_children(&mut self) {
        for i in 0..self.configs.len() {
            if !self.is_panel_visible(i) {
                continue;
            }
            if let Some(child) = self.group.child(i) {
                let cs = child.bounds();
                if cs.w() == 0 || cs.h() == 0 {
                    continue;
                }
            }
            self.group.blit_child(i);
        }
    }

    fn dispatch_key(&mut self, key: &KeyEvent) -> Option<HandleResult> {
        let km = self.keymap.clone();
        if let Some(r) = self.dispatch_panel_keys(key, &km) {
            return Some(r);
        }
        if let Some(r) = self.dispatch_resize_keys(key, &km) {
            return Some(r);
        }
        if let Some(r) = self.dispatch_tab_keys(key, &km) {
            return Some(r);
        }
        self.dispatch_alt_digit(key)
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
                panel.bar_mut().open_dropdown();
            }
            return Some(HandleResult::Consumed);
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

    fn dispatch_alt_digit(&mut self, key: &KeyEvent) -> Option<HandleResult> {
        if !key.modifiers().alt() || key.modifiers().ctrl() || key.modifiers().shift() {
            return None;
        }
        let KeyCode::Char(c) = key.code() else {
            return None;
        };
        let n = c.to_digit(10)?;
        if n < 1 {
            return None;
        }
        if let Some(panel) = self.panel_mut(self.group.focused_index()) {
            panel.activate_by_label(n as usize);
        }
        Some(HandleResult::Consumed)
    }

    pub(crate) fn find_panel_by_position(&self, pos: PanelPosition) -> Option<usize> {
        self.configs.iter().position(|c| c.position == pos)
    }

    fn is_panel_visible(&self, i: usize) -> bool {
        if self.hidden[i] && self.zoomed != Some(i) {
            return false;
        }
        if self.zoomed.is_some() && self.zoomed != Some(i) {
            return false;
        }
        true
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
