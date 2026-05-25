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
        if b.w == 0 || b.h == 0 {
            return;
        }
        // Row 0: transparent so parent's chrome shows through
        let transparent = Style {
            fg: Color::Transparent,
            bg: Color::Transparent,
            ..Style::default()
        };
        for col in 0..b.w {
            self.group.buffer_mut().put(col, 0, ' ', transparent);
        }
        // Content area: opaque fill
        for row in 1..b.h {
            for col in 0..b.w {
                self.group.buffer_mut().put(col, row, ' ', Style::default());
            }
        }

        // Draw bar (child 0)
        let buf_ptr = self.group.buffer_mut() as *mut Buffer;
        if let Some(bar) = self.group.child_mut(0) {
            bar.draw();
            unsafe { (*buf_ptr).blit(bar.buffer(), 0, 0) };
        }

        // Draw active content child
        let active_gi = self.bar().active_index() + 1;
        if let Some(child) = self.group.child_mut(active_gi) {
            child.draw();
            let cb = child.bounds();
            if cb.w > 0 && cb.h > 0 {
                let dx = cb.x.saturating_sub(b.x);
                let dy = cb.y.saturating_sub(b.y);
                unsafe { (*buf_ptr).blit(child.buffer(), dx, dy) };
            }
        }

        if self.bar().dropdown_open() {
            self.draw_dropdown_overlay();
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // Tick: broadcast to ALL children (background tabs need updates)
        if matches!(event, Event::Tick) {
            for i in 0..self.group.child_count() {
                if let Some(child) = self.group.child_mut(i) {
                    child.handle(event);
                }
            }
            // Sync active view's subtitle into tab title
            self.sync_subtitle();
            return HandleResult::Ignored;
        }
        // Three-phase dispatch: bar (preprocess) → active tab (focused) → postprocess
        let prev_active = self.bar().active_index();
        let result = self.group.dispatch(event);
        // If bar changed active tab, sync layout
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
