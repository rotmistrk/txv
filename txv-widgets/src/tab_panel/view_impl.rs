//! View trait implementation for TabPanel using GroupState.

use txv_core::prelude::*;

use super::TabPanel;
use crate::dropdown_menu::{CM_DROPDOWN_CANCELLED, CM_DROPDOWN_DONE};
use crate::tab_bar::mac_option_digit;

impl View for TabPanel {
    delegate_group_state!(group, override { set_bounds, draw, handle, select, unselect, as_any_mut });

    fn set_bounds(&mut self, r: Rect) {
        self.group.set_bounds(r);
        self.relayout();
    }

    fn select(&mut self) {
        self.group.set_focused(true);
        self.group.mark_dirty();
        self.bar_mut().set_focused(true);
        let gi = self.bar().active_index() + 1;
        if let Some(child) = self.group.child_mut(gi) {
            child.select();
        }
    }

    fn unselect(&mut self) {
        self.group.set_focused(false);
        self.group.mark_dirty();
        self.bar_mut().set_focused(false);
        self.close_dropdown();
        let gi = self.bar().active_index() + 1;
        if let Some(child) = self.group.child_mut(gi) {
            child.unselect();
        }
    }

    fn draw(&mut self) {
        let b = self.group.bounds();
        if b.w() == 0 || b.h() == 0 {
            return;
        }
        self.fill_background(b);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Command { id, data, .. } = event {
            if let Some(result) = self.handle_dropdown_cmd(*id, data) {
                return result;
            }
        }
        if let Event::Key(key) = event {
            if self.dropdown_active {
                if let Some(result) = self.handle_dropdown_digit(key) {
                    return result;
                }
            }
            if let Some(result) = self.handle_dropdown_open(key) {
                return result;
            }
        }
        if matches!(event, Event::Tick) {
            for i in 0..self.group.child_count() {
                if let Some(child) = self.group.child_mut(i) {
                    child.handle(event);
                }
            }
            self.sync_subtitle();
            return HandleResult::Ignored;
        }
        let prev_active = self.bar().active_index();
        let result = self.group.dispatch(event);
        if self.bar().active_index() != prev_active {
            self.sync_focus_from_bar(prev_active);
        }
        result
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}

impl TabPanel {
    /// Intercept dropdown commands only if our dropdown is active.
    fn handle_dropdown_cmd(
        &mut self,
        id: txv_core::event::CommandId,
        data: &Option<Box<dyn std::any::Any + Send>>,
    ) -> Option<HandleResult> {
        if !self.dropdown_active {
            return None;
        }
        if id == CM_DROPDOWN_DONE {
            let display_idx = data
                .as_ref()
                .and_then(|d| d.downcast_ref::<usize>())
                .copied()
                .unwrap_or(0);
            let tab_idx = self.dropdown_order.get(display_idx).copied().unwrap_or(display_idx);
            self.close_dropdown();
            self.set_active(tab_idx);
            return Some(HandleResult::Consumed);
        }
        if id == CM_DROPDOWN_CANCELLED {
            self.close_dropdown();
            return Some(HandleResult::Consumed);
        }
        None
    }

    /// When dropdown is open, Alt-digit/Option-digit closes it and activates tab.
    fn handle_dropdown_digit(&mut self, key: &KeyEvent) -> Option<HandleResult> {
        let n = if key.modifiers().alt() && !key.modifiers().ctrl() && !key.modifiers().shift() {
            if let KeyCode::Char(c) = key.code() {
                c.to_digit(10).map(|d| d as usize)
            } else {
                None
            }
        } else if key.modifiers() == KeyMod::NONE {
            if let KeyCode::Char(c) = key.code() {
                mac_option_digit(c)
            } else {
                None
            }
        } else {
            None
        };
        let n = n?;
        if n == 0 {
            return None;
        }
        // Use bar's own activate logic (handles both Static and LRU numbering)
        self.close_dropdown();
        self.bar_mut().activate_by_number(n);
        let idx = self.bar().active_index();
        self.set_active(idx);
        Some(HandleResult::Consumed)
    }

    /// Open dropdown on Alt-0/Down/Up or Ctrl+Shift+Down/Up.
    fn handle_dropdown_open(&mut self, key: &KeyEvent) -> Option<HandleResult> {
        if self.dropdown_active || self.tab_count() <= 1 {
            return None;
        }
        let alt_open = key.modifiers().alt()
            && !key.modifiers().ctrl()
            && matches!(key.code(), KeyCode::Char('0') | KeyCode::Down | KeyCode::Up);
        let cs_open =
            key.modifiers().ctrl() && key.modifiers().shift() && matches!(key.code(), KeyCode::Down | KeyCode::Up);
        if alt_open || cs_open {
            self.open_dropdown();
            return Some(HandleResult::Consumed);
        }
        None
    }

    fn fill_background(&mut self, b: Rect) {
        let transparent = Style::new(Color::Transparent, Color::Transparent);
        for col in 0..b.w() {
            self.group.buffer_mut().put(col, 0, ' ', transparent);
        }
        for row in 1..b.h() {
            for col in 0..b.w() {
                self.group.buffer_mut().put(col, row, ' ', Style::default());
            }
        }
    }
}
