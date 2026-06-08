//! View trait implementation for SplitPanel using GroupState.

use txv_core::cell::Color;
use txv_core::glyphs::glyphs;
use txv_core::palette::palette;
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
        if b.w() == 0 || b.h() == 0 {
            return;
        }
        let transparent = Style::new(Color::Transparent, Color::Transparent);
        self.group.buffer_mut().fill(' ', transparent);
        self.draw_dividers(b);

        for i in 0..self.group.child_count() {
            if let Some(child) = self.group.child_mut(i) {
                child.render();
            }
            self.group.blit_child(i);
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if matches!(event, Event::Tick) {
            for i in 0..self.group.child_count() {
                if let Some(child) = self.group.child_mut(i) {
                    child.handle(event);
                }
            }
            return HandleResult::Ignored;
        }
        self.group.dispatch(event)
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}

impl SplitPanel {
    fn draw_dividers(&mut self, b: Rect) {
        if self.group.child_count() <= 1 {
            return;
        }
        let dim = palette().style(StyleId::Dim);
        let g = glyphs();
        for i in 0..self.group.child_count() - 1 {
            let Some(child) = self.group.child(i) else {
                continue;
            };
            let (ox, oy) = self.group.child_origin(i);
            let cs = child.bounds();
            match self.direction {
                SplitDir::Horizontal => {
                    let x = ox + cs.w();
                    let y0 = if self.chrome_row {
                        1
                    } else {
                        0
                    };
                    self.group
                        .buffer_mut()
                        .vline(x, y0, b.h().saturating_sub(y0), g.ui().separator_v(), dim);
                }
                SplitDir::Vertical => {
                    let y = oy + cs.h();
                    self.group.buffer_mut().hline(0, y, b.w(), g.ui().separator_h(), dim);
                }
            }
        }
    }
}
