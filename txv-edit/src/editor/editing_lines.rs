//! Line-level editing operations (join, toggle case, replace char).

use super::Editor;

impl Editor {
    pub(super) fn join_lines(&mut self) {
        if self.cursor_line + 1 >= self.buf().line_count() {
            return;
        }
        let line_len = self.buf().line_len(self.cursor_line);
        let end_offset = self.buf().line_col_to_offset(self.cursor_line, line_len).unwrap_or(0);
        self.buf().delete(end_offset, end_offset + 1);
        let next_line = self.buf().line(self.cursor_line).unwrap_or_default();
        let after_join = &next_line[line_len..];
        let ws_count = after_join.chars().take_while(|c| c.is_whitespace()).count();
        if ws_count > 0 {
            let ws_start = self.buf().line_col_to_offset(self.cursor_line, line_len).unwrap_or(0);
            let ws_end = self
                .buf()
                .line_col_to_offset(self.cursor_line, line_len + ws_count)
                .unwrap_or(ws_start);
            self.buf().delete(ws_start, ws_end);
        }
        let offset = self.buf().line_col_to_offset(self.cursor_line, line_len);
        if let Some(offset) = offset {
            self.buf().insert(offset, " ");
        }
    }

    pub(super) fn toggle_case(&mut self) {
        let line = self.buf().line(self.cursor_line).unwrap_or_default();
        if let Some(ch) = line.chars().nth(self.cursor_col) {
            let toggled = if ch.is_uppercase() {
                ch.to_lowercase().next().unwrap_or(ch)
            } else {
                ch.to_uppercase().next().unwrap_or(ch)
            };
            let offset = self.buf().line_col_to_offset(self.cursor_line, self.cursor_col);
            if let Some(offset) = offset {
                self.buf().delete(offset, offset + ch.len_utf8());
                self.buf().insert(offset, &toggled.to_string());
            }
            self.cursor_col += 1;
            self.clamp_col();
        }
    }

    pub(super) fn replace_char(&mut self, ch: char) {
        let offset = self.buf().line_col_to_offset(self.cursor_line, self.cursor_col);
        if let Some(offset) = offset {
            let content = self.buf().content();
            let old_len = content[offset..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            self.buf().delete(offset, offset + old_len);
            self.buf().insert(offset, &ch.to_string());
        }
    }
}
