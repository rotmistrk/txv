//! View trait implementation for SplitPanel using GroupState.

use txv_core::prelude::*;

use super::{SplitDir, SplitPanel};

impl View for SplitPanel {
    delegate_group_state!(group, override { set_bounds, draw, handle, as_any_mut });

    fn set_bounds(&mut self, r: Rect) {
        self.group.set_bounds(r);
        self.relayout();
    }

    fn draw(&mut self) {
        let b = self.group.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        let transparent = Style {
            fg: txv_core::cell::Color::Transparent,
            bg: txv_core::cell::Color::Transparent,
            ..Style::default()
        };
        self.group.buffer_mut().fill(' ', transparent);

        // Draw dividers BEFORE children so tab bars render on top
        if self.group.child_count() > 1 {
            let dim = txv_core::palette::palette().style(StyleId::Dim);
            let g = txv_core::glyphs::glyphs();
            for i in 0..self.group.child_count() - 1 {
                let Some(child) = self.group.child(i) else {
                    continue;
                };
                let cb = child.bounds();
                match self.direction {
                    SplitDir::Horizontal => {
                        let x = (cb.x + cb.w).saturating_sub(b.x);
                        let y0 = if self.chrome_row {
                            1
                        } else {
                            0
                        };
                        self.group
                            .buffer_mut()
                            .vline(x, y0, b.h.saturating_sub(y0), g.ui.separator_v, dim);
                    }
                    SplitDir::Vertical => {
                        let y = (cb.y + cb.h).saturating_sub(b.y);
                        self.group.buffer_mut().hline(0, y, b.w, g.ui.separator_h, dim);
                    }
                }
            }
        }

        let buf_ptr = self.group.buffer_mut() as *mut Buffer;
        for i in 0..self.group.child_count() {
            if let Some(child) = self.group.child_mut(i) {
                child.draw();
                let cb = child.bounds();
                if cb.w > 0 && cb.h > 0 {
                    let dx = cb.x.saturating_sub(b.x);
                    let dy = cb.y.saturating_sub(b.y);
                    unsafe { (*buf_ptr).blit(child.buffer(), dx, dy) };
                }
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // Tick: broadcast to ALL children
        if matches!(event, Event::Tick) {
            for i in 0..self.group.child_count() {
                if let Some(child) = self.group.child_mut(i) {
                    child.handle(event);
                }
            }
            return HandleResult::Ignored;
        }
        // All other events: three-phase dispatch
        self.group.dispatch(event)
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}
