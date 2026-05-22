//! View trait implementation for TiledWorkspace.

use txv_core::prelude::*;

use super::types::{PanelPosition, SplitDir};
use super::TiledWorkspace;

impl View for TiledWorkspace {
    delegate_group_state!(group, override { set_bounds, draw, handle });

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
        let my_bounds = self.group.bounds();

        for i in 0..self.configs.len() {
            if !self.is_panel_visible(i) {
                continue;
            }
            if let Some(child) = self.group.child_mut(i) {
                child.draw();
            }
        }

        let buf_ptr = self.group.buffer_mut() as *mut Buffer;
        for i in 0..self.configs.len() {
            if !self.is_panel_visible(i) {
                continue;
            }
            if let Some(child) = self.group.child(i) {
                let cb = child.bounds();
                if cb.w == 0 || cb.h == 0 {
                    continue;
                }
                let dx = cb.x.saturating_sub(my_bounds.x);
                let dy = cb.y.saturating_sub(my_bounds.y);
                unsafe { (*buf_ptr).blit(child.buffer(), dx, dy) };
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // Handle command events
        if let Event::Command { id, data } = event {
            if self.handle_command(*id, data) {
                return HandleResult::Consumed;
            }
        }

        // Tick goes to all visible panels
        if matches!(event, Event::Tick) {
            for i in 0..self.configs.len() {
                if !self.hidden[i] {
                    if let Some(child) = self.group.child_mut(i) {
                        child.handle(event);
                    }
                }
            }
            return HandleResult::Ignored;
        }

        let Event::Key(key) = event else {
            return self.group.dispatch(event);
        };

        // Skip key handling if disabled (app/status bar owns keys)
        if !self.handle_keys {
            return self.group.dispatch(event);
        }

        // Key dispatch → internal method calls
        let km = self.keymap.clone();

        if km.matches(key, &km.toggle_tree) {
            if let Some(id) = self.find_panel_by_position(PanelPosition::Left) {
                self.toggle_panel(id);
                return HandleResult::Consumed;
            }
        }
        if km.matches(key, &km.toggle_tools) {
            let id = self
                .find_panel_by_position(PanelPosition::Right)
                .or_else(|| self.find_panel_by_position(PanelPosition::Bottom));
            if let Some(id) = id {
                self.toggle_panel(id);
                return HandleResult::Consumed;
            }
        }
        if km.matches(key, &km.zoom) {
            self.toggle_zoom();
            return HandleResult::Consumed;
        }
        if km.matches(key, &km.focus_left) {
            self.focus_direction(-1, 0);
            return HandleResult::Consumed;
        }
        if km.matches(key, &km.focus_right) {
            self.focus_direction(1, 0);
            return HandleResult::Consumed;
        }
        if km.matches(key, &km.focus_up) {
            self.focus_direction(0, -1);
            return HandleResult::Consumed;
        }
        if km.matches(key, &km.focus_down) {
            self.focus_direction(0, 1);
            return HandleResult::Consumed;
        }
        if km.matches(key, &km.resize_left) {
            self.resize_panel(SplitDir::Horizontal, -1);
            return HandleResult::Consumed;
        }
        if km.matches(key, &km.resize_right) {
            self.resize_panel(SplitDir::Horizontal, 1);
            return HandleResult::Consumed;
        }
        if km.matches(key, &km.resize_up) {
            self.resize_panel(SplitDir::Vertical, -1);
            return HandleResult::Consumed;
        }
        if km.matches(key, &km.resize_down) {
            self.resize_panel(SplitDir::Vertical, 1);
            return HandleResult::Consumed;
        }
        if km.matches(key, &km.tab_dropdown) {
            if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                panel.open_dropdown();
            }
            return HandleResult::Consumed;
        }
        if km.matches(key, &km.layout_cycle) {
            self.cycle_layout();
            return HandleResult::Consumed;
        }

        // Alt+digit for tab switching
        if key.modifiers.alt && !key.modifiers.ctrl && !key.modifiers.shift {
            if let KeyCode::Char(c) = key.code {
                if let Some(n) = c.to_digit(10) {
                    if n >= 1 {
                        if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                            let idx = (n as usize).saturating_sub(1);
                            if idx < panel.tab_count() {
                                panel.set_active(idx);
                            }
                        }
                        return HandleResult::Consumed;
                    }
                }
            }
        }

        self.group.dispatch(event)
    }
}

impl TiledWorkspace {
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
        let current = self.group.focused_index();
        let cur_bounds = self.group.child(current).map(|c| c.bounds()).unwrap_or_default();
        let cur_cx = cur_bounds.x as i32 + cur_bounds.w as i32 / 2;
        let cur_cy = cur_bounds.y as i32 + cur_bounds.h as i32 / 2;

        let mut best: Option<(usize, i32)> = None;
        for i in 0..self.configs.len() {
            if i == current || self.hidden[i] {
                continue;
            }
            let b = self.group.child(i).map(|c| c.bounds()).unwrap_or_default();
            if b.w == 0 || b.h == 0 {
                continue;
            }
            let cx = b.x as i32 + b.w as i32 / 2;
            let cy = b.y as i32 + b.h as i32 / 2;

            let in_direction = (dx > 0 && cx > cur_cx)
                || (dx < 0 && cx < cur_cx)
                || (dy > 0 && cy > cur_cy)
                || (dy < 0 && cy < cur_cy);
            if !in_direction {
                continue;
            }
            let dist = (cx - cur_cx).abs() + (cy - cur_cy).abs();
            if best.is_none() || dist < best.unwrap().1 {
                best = Some((i, dist));
            }
        }
        if let Some((target, _)) = best {
            self.group.switch_focus(target);
        }
    }
}
