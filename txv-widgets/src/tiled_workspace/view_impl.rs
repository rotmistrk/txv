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
                let cs = child.bounds();
                if cs.w == 0 || cs.h == 0 {
                    continue;
                }
                let (ox, oy) = self.group.child_origin(i);
                unsafe { (*buf_ptr).blit(child.buffer(), ox, oy) };
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // Handle command events
        if let Event::Command { id, data, .. } = event {
            if self.handle_command(*id, data) {
                return HandleResult::Consumed;
            }
        }

        // Tick is broadcast to ALL children (state update, not rendering)
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
                panel.bar_mut().open_dropdown();
            }
            return HandleResult::Consumed;
        }
        if km.matches(key, &km.layout_cycle) {
            self.cycle_layout();
            return HandleResult::Consumed;
        }
        if km.matches(key, &km.tab_next) {
            if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                panel.tab_next();
            }
            return HandleResult::Consumed;
        }
        if km.matches(key, &km.tab_prev) {
            if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                panel.tab_prev();
            }
            return HandleResult::Consumed;
        }
        if km.matches(key, &km.subpanel_move_tab) {
            self.move_tab_to_subpanel();
            return HandleResult::Consumed;
        }

        // Alt+digit for tab switching
        if key.modifiers.alt && !key.modifiers.ctrl && !key.modifiers.shift {
            if let KeyCode::Char(c) = key.code {
                if let Some(n) = c.to_digit(10) {
                    if n >= 1 {
                        if let Some(panel) = self.panel_mut(self.group.focused_index()) {
                            panel.activate_by_label(n as usize);
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
        // In zoom mode, cycle through visible panels and update zoom
        if self.zoomed.is_some() {
            if dx > 0 || dy > 0 {
                self.focus_next_visible();
            } else {
                self.focus_prev_visible();
            }
            self.zoomed = Some(self.group.focused_index());
            self.recompute_layout();
            return;
        }

        let current = self.group.focused_index();
        let visible: Vec<usize> = (0..self.configs.len())
            .filter(|&i| !self.hidden[i])
            .filter(|&i| {
                self.group
                    .child(i)
                    .map(|c| c.bounds().w > 0 && c.bounds().h > 0)
                    .unwrap_or(false)
            })
            .collect();
        if visible.len() <= 1 {
            return;
        }
        let pos = visible.iter().position(|&i| i == current).unwrap_or(0);
        let forward = dx > 0 || dy > 0;
        let next = if forward {
            visible[(pos + 1) % visible.len()]
        } else {
            visible[(pos + visible.len() - 1) % visible.len()]
        };
        self.group.switch_focus(next);
    }
}
