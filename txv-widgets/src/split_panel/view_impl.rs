use txv_core::prelude::*;

use crate::tiled_workspace::types::SplitDir;

use super::SplitPanel;

impl View for SplitPanel {
    delegate_view_state!(state, override { set_bounds, set_sink, select, unselect, draw, handle, needs_redraw, mark_redrawn, as_any_mut });

    fn needs_redraw(&self) -> bool {
        self.state.is_dirty() || self.children.iter().any(|c| c.needs_redraw())
    }

    fn mark_redrawn(&mut self) {
        self.state.mark_redrawn();
        for child in &mut self.children {
            child.mark_redrawn();
        }
    }

    fn set_bounds(&mut self, r: Rect) {
        self.state.set_bounds(r);
        self.relayout();
    }

    fn set_sink(&mut self, sink: EventSink) {
        self.state.set_sink(sink.clone());
        for child in &mut self.children {
            child.set_sink(sink.clone());
        }
    }

    fn select(&mut self) {
        self.state.set_focused(true);
        if let Some(child) = self.children.get_mut(self.focused) {
            child.select();
        }
    }

    fn unselect(&mut self) {
        self.state.set_focused(false);
        if let Some(child) = self.children.get_mut(self.focused) {
            child.unselect();
        }
    }

    fn draw(&mut self) {
        let b = self.state.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        let transparent = Style {
            fg: txv_core::cell::Color::Transparent,
            bg: txv_core::cell::Color::Transparent,
            ..Style::default()
        };
        self.state.buffer_mut().fill(' ', transparent);

        // Draw dividers BEFORE children so tab bars overlay them via transparency
        if self.children.len() > 1 {
            let dim = txv_core::palette::palette().base.dim.to_style();
            let g = txv_core::glyphs::glyphs();
            for i in 0..self.children.len() - 1 {
                let cb = self.children[i].bounds();
                match self.direction {
                    SplitDir::Horizontal => {
                        let x = (cb.x + cb.w).saturating_sub(b.x);
                        let y0 = if self.chrome_row {
                            1
                        } else {
                            0
                        };
                        self.state
                            .buffer_mut()
                            .vline(x, y0, b.h.saturating_sub(y0), g.ui.separator_v, dim);
                    }
                    SplitDir::Vertical => {
                        let y = (cb.y + cb.h).saturating_sub(b.y);
                        self.state.buffer_mut().hline(0, y, b.w, g.ui.separator_h, dim);
                    }
                }
            }
        }

        let buf_ptr = self.state.buffer_mut() as *mut Buffer;
        for child in &mut self.children {
            child.draw();
            let cb = child.bounds();
            if cb.w > 0 && cb.h > 0 {
                let dx = cb.x.saturating_sub(b.x);
                let dy = cb.y.saturating_sub(b.y);
                unsafe { (*buf_ptr).blit(child.buffer(), dx, dy) };
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if matches!(event, Event::Tick) {
            for child in &mut self.children {
                child.handle(event);
            }
            return HandleResult::Ignored;
        }
        if let Some(child) = self.children.get_mut(self.focused) {
            return child.handle(event);
        }
        HandleResult::Ignored
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn cursor(&self) -> Option<txv_core::cursor::CursorRequest> {
        let child = self.children.get(self.focused)?;
        let mut req = child.cursor()?;
        let cb = child.bounds();
        let b = self.state.bounds();
        req.x = req.x.saturating_add(cb.x).saturating_sub(b.x);
        req.y = req.y.saturating_add(cb.y).saturating_sub(b.y);
        Some(req)
    }
}
