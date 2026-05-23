//! View trait implementation for TabPanel.

use txv_core::prelude::*;

use super::TabPanel;

impl View for TabPanel {
    delegate_view_state!(state, override { set_bounds, set_sink, draw, handle, select, unselect });

    fn set_bounds(&mut self, r: Rect) {
        self.state.set_bounds(r);
        self.relayout();
    }

    fn set_sink(&mut self, sink: EventSink) {
        self.state.set_sink(sink.clone());
        self.bar.set_sink(sink.clone());
        for child in &mut self.children {
            child.set_sink(sink.clone());
        }
    }

    fn select(&mut self) {
        self.state.set_focused(true);
        self.bar.set_focused(true);
        self.state.mark_dirty();
        if let Some(child) = self.children.get_mut(self.bar.active_index()) {
            child.select();
        }
    }

    fn unselect(&mut self) {
        self.state.set_focused(false);
        self.bar.set_focused(false);
        self.bar.close_dropdown();
        self.state.mark_dirty();
        if let Some(child) = self.children.get_mut(self.bar.active_index()) {
            child.unselect();
        }
    }

    fn draw(&mut self) {
        let b = self.state.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        // Row 0: transparent so parent's chrome background shows through
        let transparent = Style {
            fg: Color::Transparent,
            bg: Color::Transparent,
            ..Style::default()
        };
        for col in 0..b.w {
            self.state.buffer_mut().put(col, 0, ' ', transparent);
        }
        // Content area: opaque fill
        for row in 1..b.h {
            for col in 0..b.w {
                self.state.buffer_mut().put(col, row, ' ', Style::default());
            }
        }

        self.bar.draw();
        let bar_buf = self.bar.buffer();
        let buf_ptr = self.state.buffer_mut() as *mut Buffer;
        unsafe { (*buf_ptr).blit(bar_buf, 0, 0) };

        let active = self.bar.active_index();
        if let Some(child) = self.children.get_mut(active) {
            child.draw();
            let cb = child.bounds();
            if cb.w > 0 && cb.h > 0 {
                let dx = cb.x.saturating_sub(b.x);
                let dy = cb.y.saturating_sub(b.y);
                unsafe { (*buf_ptr).blit(child.buffer(), dx, dy) };
            }
        }

        if self.bar.dropdown_open() {
            self.draw_dropdown_overlay();
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if matches!(event, Event::Tick) {
            for child in &mut self.children {
                child.handle(event);
            }
            return HandleResult::Ignored;
        }
        let prev_active = self.bar.active_index();
        let result = self.bar.handle(event);
        if result == HandleResult::Consumed {
            if self.bar.active_index() != prev_active {
                self.relayout();
            }
            return HandleResult::Consumed;
        }
        let active = self.bar.active_index();
        if let Some(child) = self.children.get_mut(active) {
            return child.handle(event);
        }
        HandleResult::Ignored
    }
}
