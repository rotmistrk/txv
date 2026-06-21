//! View trait implementation for InputLine.

use std::sync::Arc;

use txv_core::cursor::{CursorRequest, CursorShape};
use txv_core::prelude::*;

use super::InputLine;

impl View for InputLine {
    delegate_view_state!(state, override { cursor, select, as_any_mut });

    fn select(&mut self) {
        self.state.set_focused(true);
        self.state.mark_dirty();
        self.select_all();
    }

    fn cursor(&self) -> Option<CursorRequest> {
        if !self.state.is_focused() {
            return None;
        }
        let w = self.state.bounds().w() as usize;
        let start = self.visible_start(w);
        Some(CursorRequest::new((self.cursor - start) as u16, 0, CursorShape::Bar))
    }

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        if w == 0 || self.state.buffer_mut().height() == 0 {
            return;
        }
        let style = self.resolve_style(StyleId::Text);
        let sel_style = self.resolve_style(StyleId::EditSelection);
        let ww = w as usize;
        let start = self.visible_start(ww);
        self.state.buffer_mut().hline(0, 0, w, ' ', style);
        let sel_range = self.selection_range();
        for (i, ch) in self.text.chars().enumerate().skip(start).take(ww) {
            let x = (i - start) as u16;
            let in_sel = sel_range.is_some_and(|(lo, hi)| i >= lo && i < hi);
            let s = if in_sel {
                sel_style
            } else {
                style
            };
            let display_ch = if self.password {
                '*'
            } else {
                ch
            };
            self.state.buffer_mut().put(x, 0, display_ch, s);
        }
        // Overflow indicators
        let total_chars = self.char_count();
        if ww > 0 && total_chars > ww {
            let ov_fg = self.resolve_style(StyleId::OverflowIndicator).fg();
            if start > 0 {
                self.state.buffer_mut().put(0, 0, '…', style.with_fg(ov_fg));
            }
            if start + ww < total_chars {
                let rx = (ww - 1) as u16;
                self.state.buffer_mut().put(rx, 0, '…', style.with_fg(ov_fg));
            }
        }
    }

    fn set_palette(&mut self, palette: Arc<dyn Palette>) {
        self.palette = Some(palette);
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        self.handle_event(event)
    }
}

impl Default for InputLine {
    fn default() -> Self {
        Self::new()
    }
}
