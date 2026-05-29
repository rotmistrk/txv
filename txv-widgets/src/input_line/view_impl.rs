//! View trait implementation for InputLine.

use std::sync::Arc;

use txv_core::prelude::*;

use super::InputLine;

impl View for InputLine {
    delegate_view_state!(state, override { cursor, select, as_any_mut });

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
            self.state.buffer_mut().put(x, 0, ch, s);
        }
        // Overflow indicators
        let total_chars = self.char_count();
        if ww > 0 && total_chars > ww {
            let ov_fg = self.resolve_style(StyleId::OverflowIndicator).fg;
            if start > 0 {
                self.state.buffer_mut().put(0, 0, '…', Style { fg: ov_fg, ..style });
            }
            if start + ww < total_chars {
                let rx = (ww - 1) as u16;
                self.state.buffer_mut().put(rx, 0, '…', Style { fg: ov_fg, ..style });
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
        if let Event::Command { data, .. } = event {
            return self.handle_command(data);
        }
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        let shift = key.modifiers.shift;
        match &key.code {
            KeyCode::Char(ch) => {
                if key.modifiers.alt || key.modifiers.ctrl {
                    return HandleResult::Ignored;
                }
                self.handle_char(*ch);
                self.update_completions();
            }
            KeyCode::Backspace => {
                self.handle_backspace();
                self.update_completions();
            }
            KeyCode::Delete => {
                self.handle_delete();
                self.update_completions();
            }
            KeyCode::Left => {
                let new = self.cursor.saturating_sub(1);
                if new != self.cursor {
                    self.handle_nav(shift, new);
                }
            }
            KeyCode::Right => {
                let max = self.char_count();
                let new = (self.cursor + 1).min(max);
                if new != self.cursor {
                    self.handle_nav(shift, new);
                }
            }
            KeyCode::Home => self.handle_nav(shift, 0),
            KeyCode::End => self.handle_nav(shift, self.char_count()),
            KeyCode::Up => {
                if self.sidekick_visible {
                    self.sidekick.select_prev();
                } else {
                    self.handle_history_up();
                }
            }
            KeyCode::Down => {
                if self.sidekick_visible {
                    self.sidekick.select_next();
                } else {
                    self.handle_history_down();
                }
            }
            KeyCode::Tab => {
                if self.sidekick_visible {
                    self.apply_sidekick_selection();
                } else {
                    self.try_complete();
                }
            }
            KeyCode::Enter => {
                self.hide_sidekick();
                self.push_history();
                self.state
                    .put_command(self.submit_command, Some(Box::new(self.text.clone())));
            }
            KeyCode::Esc => {
                if self.sidekick_visible {
                    self.hide_sidekick();
                } else {
                    self.state.put_command(CM_CANCEL, None);
                }
            }
            _ => return HandleResult::Ignored,
        }
        HandleResult::Consumed
    }
}
