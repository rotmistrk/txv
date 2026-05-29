//! InlineEditor rendering — draw, overflow indicators, completion.

use txv_core::prelude::*;

use super::InlineEditor;

impl InlineEditor {
    /// Draw the editor at the given position on the buffer.
    pub fn draw(&mut self, buf: &mut Buffer, x: u16, y: u16, width: u16, style: Style) {
        let w = width as usize;
        let char_cursor = self.buffer[..self.cursor].chars().count();
        let total_chars = self.buffer.chars().count();
        // Adjust scroll
        if total_chars <= w {
            self.scroll_offset = 0;
        } else if char_cursor < self.scroll_offset {
            self.scroll_offset = char_cursor;
        } else if w > 0 && char_cursor >= self.scroll_offset + w {
            self.scroll_offset = char_cursor - w + 1;
        }
        let pal = palette();
        let inherit_bg = style.bg == Color::Reset;
        let sel_bg = pal.style(StyleId::EditSelection).bg;
        let sel_style = Style {
            bg: if sel_bg != Color::Reset {
                sel_bg
            } else {
                style.bg
            },
            ..style
        };
        let cursor_style = if !inherit_bg {
            Style {
                fg: style.bg,
                bg: style.fg,
                ..style
            }
        } else {
            pal.style(StyleId::InputCursor)
        };
        let sel = self.selection_range();
        if !inherit_bg {
            buf.hline(x, y, width, ' ', style);
        } else {
            // Clear text area preserving existing bg
            for i in 0..width {
                let bg = buf.cell(x + i, y).style.bg;
                buf.put(x + i, y, ' ', Style { bg, ..style });
            }
        }
        // Render chars starting from scroll_offset
        let mut byte_pos = 0;
        for (ci, ch) in self.buffer.chars().enumerate() {
            if ci >= self.scroll_offset + w {
                break;
            }
            if ci >= self.scroll_offset {
                let vi = ci - self.scroll_offset;
                let mut st = if byte_pos == self.cursor {
                    cursor_style
                } else if sel.is_some_and(|(s, e)| byte_pos >= s && byte_pos < e) {
                    sel_style
                } else {
                    style
                };
                if inherit_bg && st.bg == Color::Reset {
                    st.bg = buf.cell(x + vi as u16, y).style.bg;
                }
                buf.put(x + vi as u16, y, ch, st);
            }
            byte_pos += ch.len_utf8();
        }
        // Cursor at end of text
        let visible_cursor = char_cursor.saturating_sub(self.scroll_offset);
        if self.cursor >= self.buffer.len() && visible_cursor < w {
            let mut cs = cursor_style;
            if inherit_bg && cs.bg == Color::Reset {
                cs.bg = buf.cell(x + visible_cursor as u16, y).style.bg;
            }
            buf.put(x + visible_cursor as u16, y, ' ', cs);
        }
        // Overflow indicators
        if w > 0 && total_chars > w {
            let mut ov = Style {
                fg: pal.style(StyleId::OverflowIndicator).fg,
                ..style
            };
            if self.scroll_offset > 0 {
                if inherit_bg {
                    ov.bg = buf.cell(x, y).style.bg;
                }
                buf.put(x, y, '…', ov);
            }
            if self.scroll_offset + w < total_chars {
                let rx = x + (w - 1) as u16;
                if inherit_bg {
                    ov.bg = buf.cell(rx, y).style.bg;
                }
                buf.put(rx, y, '…', ov);
            }
        }
    }

    /// Apply tab completion: cycle through candidates.
    pub fn apply_completion(&mut self, candidates: &[String], direction: i32) {
        if candidates.is_empty() {
            return;
        }
        let idx = candidates
            .iter()
            .position(|c| c == &self.buffer)
            .map(|i| {
                if direction > 0 {
                    (i + 1) % candidates.len()
                } else {
                    (i + candidates.len() - 1) % candidates.len()
                }
            })
            .unwrap_or(0);
        if let Some(text) = candidates.get(idx) {
            self.buffer = text.clone();
            self.cursor = self.buffer.len();
        }
    }

    /// Return a cursor request relative to the draw origin (x, y).
    pub fn cursor_request(&self, x: u16, y: u16) -> Option<txv_core::cursor::CursorRequest> {
        Some(txv_core::cursor::CursorRequest {
            x: x + self.cursor as u16,
            y,
            shape: txv_core::cursor::CursorShape::Bar,
        })
    }
}

#[cfg(test)]
#[path = "inline_edit_draw_tests.rs"]
mod tests;
