//! View trait implementation for TabPanel using GroupState.

use txv_core::prelude::*;

use super::TabPanel;

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
        self.bar_mut().close_dropdown();
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
        if let Some(bar) = self.group.child_mut(0) {
            bar.render();
        }
        self.group.blit_child(0);
        self.draw_active_content();
        if self.bar().dropdown_open() {
            self.draw_dropdown();
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
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

    fn draw_active_content(&mut self) {
        let active_gi = self.bar().active_index() + 1;
        if let Some(child) = self.group.child_mut(active_gi) {
            child.render();
        }
        self.group.blit_child(active_gi);
    }
}
