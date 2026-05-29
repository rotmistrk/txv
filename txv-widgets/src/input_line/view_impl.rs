//! View trait implementation for InputLine.

use std::sync::Arc;

use txv_core::prelude::*;

use super::InputLine;

impl View for InputLine {
    delegate_view_state!(state, override { cursor, select });

    fn select(&mut self) {
        self.state.set_focused(true);
        self.state.mark_dirty();
        self.select_all();
    }

    fn cursor(&self) -> Option<txv_core::cursor::CursorRequest> {
        if !self.state.is_focused() {
            return None;
        }
        let w = self.state.bounds().w as usize;
        let start = self.visible_start(w);
        Some(txv_core::cursor::CursorRequest {
            x: (self.cursor - start) as u16,
            y: 0,
            shape: txv_core::cursor::CursorShape::Bar,
        })
    }

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        if w == 0 || self.state.buffer_mut().height() == 0 {
            return;
        }
        let style = self.resolve_style(StyleId::StatusBar);
        let sel_style = self.resolve_style(StyleId::EditSelection);
        self.state.buffer_mut().hline(0, 0, w, ' ', style);
        let ww = w as usize;
        let start = self.visible_start(ww);
        let sel_range = self.selection_range();
        for (i, ch) in self.text.chars().enumerate().skip(start).take(ww) {
            let x = (i - start) as u16;
            let in_sel = sel_range.is_some_and(|(lo, hi)| i >= lo && i < hi);
            let s = if in_sel {
                sel_style
            } else {
                style
            };
            self.state.buffer_mut().put(x, 0, ch, s);
        }
        if self.selection.is_none() {
            let cx = (self.cursor - start) as u16;
            if cx < w {
                let ch = self.text.chars().nth(self.cursor).unwrap_or(' ');
                let cs = self.resolve_style(StyleId::InputCursor);
                self.state.buffer_mut().put(cx, 0, ch, cs);
            }
        }
        // Overflow indicators
        let total_chars = self.text.chars().count();
        if ww > 0 && total_chars > ww {
            let ov = Style {
                fg: self.resolve_style(StyleId::OverflowIndicator).fg,
                ..style
            };
            if start > 0 {
                self.state.buffer_mut().put(0, 0, '…', ov);
            }
            if start + ww < total_chars {
                self.state.buffer_mut().put((ww - 1) as u16, 0, '…', ov);
            }
        }
    }

    fn set_palette(&mut self, palette: Arc<dyn Palette>) {
        self.palette = Some(palette);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Command { data, .. } = event {
            return self.handle_command(data);
        }
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        match &key.code {
            KeyCode::Char(ch) => self.handle_char(*ch),
            KeyCode::Backspace => self.handle_backspace(),
            KeyCode::Delete => self.handle_delete(),
            KeyCode::Left if self.cursor > 0 => {
                self.selection = None;
                self.cursor -= 1;
                self.state.mark_dirty()
            }
            KeyCode::Right if self.cursor < self.text.len() => {
                self.selection = None;
                self.cursor += 1;
                self.state.mark_dirty()
            }
            KeyCode::Home => {
                self.selection = None;
                self.cursor = 0;
                self.state.mark_dirty()
            }
            KeyCode::End => {
                self.selection = None;
                self.cursor = self.text.len();
                self.state.mark_dirty()
            }
            KeyCode::Up => self.handle_history_up(),
            KeyCode::Down => self.handle_history_down(),
            KeyCode::Tab => self.try_complete(),
            KeyCode::Enter => {
                self.push_history();
                self.state
                    .put_command(self.submit_command, Some(Box::new(self.text.clone())));
            }
            KeyCode::Esc => self.state.put_command(CM_CANCEL, None),
            _ => return HandleResult::Ignored,
        }
        HandleResult::Consumed
    }
}
