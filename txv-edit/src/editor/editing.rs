//! Text editing and clipboard methods.

use super::keymap::EditorMode;
use super::motions;
use super::Editor;

impl Editor {
    pub(super) fn enter_insert_after(&mut self) {
        self.buf().begin_group();
        self.mode = EditorMode::Insert;
        let len = self.buf().line_len(self.cursor_line);
        if self.cursor_col < len {
            self.cursor_col += 1;
        }
    }

    pub(super) fn exit_insert(&mut self) {
        // If returning from a block insert, replicate to other lines
        if let Some((sl, el, sc)) = self.pending_block_insert.take() {
            self.replicate_block_insert(sl, el, sc);
        }
        self.buf().end_group();
        self.mode = EditorMode::Normal;
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    fn replicate_block_insert(&mut self, sl: usize, el: usize, sc: usize) {
        // Extract what was typed on the first line (from sc to cursor_col)
        let inserted_len = self.cursor_col.saturating_sub(sc);
        if inserted_len == 0 {
            return;
        }
        let line = self.buf().line(sl).unwrap_or_default();
        let inserted: String = line.chars().skip(sc).take(inserted_len).collect();
        // Insert the same text at sc on lines sl+1..=el
        for line_idx in (sl + 1..=el).rev() {
            if line_idx >= self.buf().line_count() {
                continue;
            }
            let offset = self.buf().line_col_to_offset(line_idx, sc).unwrap_or(0);
            self.buf().insert(offset, &inserted);
        }
    }

    pub(super) fn open_line_below(&mut self) {
        self.buf().begin_group();
        self.mode = EditorMode::Insert;
        let indent = if self.options.autoindent() {
            self.current_line_indent()
        } else {
            String::new()
        };
        let line_len = self.buf().line_len(self.cursor_line);
        let offset = self.buf().line_col_to_offset(self.cursor_line, line_len);
        if let Some(offset) = offset {
            let text = format!("\n{indent}");
            self.buf().insert(offset, &text);
            self.cursor_line += 1;
            self.cursor_col = indent.len();
        }
    }

    pub(super) fn open_line_above(&mut self) {
        self.buf().begin_group();
        self.mode = EditorMode::Insert;
        let indent = if self.options.autoindent() {
            self.current_line_indent()
        } else {
            String::new()
        };
        let offset = self.buf().line_col_to_offset(self.cursor_line, 0);
        if let Some(offset) = offset {
            let text = format!("{indent}\n");
            self.buf().insert(offset, &text);
            self.cursor_col = indent.len();
        }
    }

    pub(super) fn insert_char(&mut self, ch: char) {
        let offset = self.buf().line_col_to_offset(self.cursor_line, self.cursor_col);
        if let Some(offset) = offset {
            if ch == '\t' {
                self.buf().insert(offset, "    ");
                self.cursor_col += 4;
            } else {
                self.buf().insert(offset, &ch.to_string());
                self.cursor_col += 1;
            }
        }
    }

    pub(super) fn insert_newline(&mut self) {
        let offset = self.buf().line_col_to_offset(self.cursor_line, self.cursor_col);
        let Some(offset) = offset else {
            return;
        };
        let indent = if self.options.autoindent() {
            self.current_line_indent()
        } else {
            String::new()
        };
        let text = format!("\n{indent}");
        self.buf().insert(offset, &text);
        self.cursor_line += 1;
        self.cursor_col = indent.len();
    }

    pub(super) fn delete_char_forward(&mut self) {
        let line_len = self.buf().line_len(self.cursor_line);
        if self.cursor_col < line_len {
            let offset = self.buf().line_col_to_offset(self.cursor_line, self.cursor_col);
            if let Some(offset) = offset {
                let content = self.buf().content();
                let ch_len = content[offset..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                self.buf().delete(offset, offset + ch_len);
            }
        }
    }

    pub(super) fn delete_char_backward(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            let offset = self.buf().line_col_to_offset(self.cursor_line, self.cursor_col);
            if let Some(offset) = offset {
                let content = self.buf().content();
                let ch_len = content[offset..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                self.buf().delete(offset, offset + ch_len);
            }
        } else if self.cursor_line > 0 {
            let prev_len = self.buf().line_len(self.cursor_line - 1);
            let offset = self.buf().line_col_to_offset(self.cursor_line, 0);
            if let Some(offset) = offset {
                self.buf().delete(offset - 1, offset);
                self.cursor_line -= 1;
                self.cursor_col = prev_len;
            }
        }
    }

    pub(super) fn delete_line(&mut self) {
        let line = self.buf().line(self.cursor_line).unwrap_or_default();
        self.yank_linewise(line);
        let line_count = self.buf().line_count();
        let start = self.buf().line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
        let end = if self.cursor_line + 1 < line_count {
            self.buf().line_col_to_offset(self.cursor_line + 1, 0).unwrap_or(start)
        } else if self.cursor_line > 0 {
            // Last line: also remove the preceding newline
            let prev_end = self.buf().line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
            let content_end = self.buf().content().len();
            // Delete from end of previous line to end of content
            let actual_start = if prev_end > 0 {
                prev_end - 1
            } else {
                0
            };
            if actual_start < content_end {
                self.buf().delete(actual_start, content_end);
            }
            self.cursor_line -= 1;
            self.clamp_col();
            return;
        } else {
            self.buf().content().len()
        };
        if start < end {
            self.buf().delete(start, end);
        }
        if self.cursor_line >= self.buf().line_count() && self.cursor_line > 0 {
            self.cursor_line -= 1;
        }
        self.clamp_col();
    }

    pub(super) fn delete_word(&mut self) {
        let start_offset = self
            .buf()
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        let (new_line, new_col) = motions::word_forward(&self.buf(), self.cursor_line, self.cursor_col);
        let end_offset = self.buf().line_col_to_offset(new_line, new_col).unwrap_or(start_offset);
        if end_offset > start_offset {
            let content = self.buf().content();
            self.yank(content[start_offset..end_offset].to_string());
            self.buf().delete(start_offset, end_offset);
            let (l, c) = self.buf().offset_to_line_col(start_offset);
            self.cursor_line = l;
            self.cursor_col = c;
        }
    }

    pub(super) fn delete_word_backward(&mut self) {
        let end_offset = self
            .buf()
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        let (new_line, new_col) = motions::word_backward(&self.buf(), self.cursor_line, self.cursor_col);
        let start_offset = self.buf().line_col_to_offset(new_line, new_col).unwrap_or(end_offset);
        if end_offset > start_offset {
            let content = self.buf().content();
            self.yank(content[start_offset..end_offset].to_string());
            self.buf().delete(start_offset, end_offset);
            self.cursor_line = new_line;
            self.cursor_col = new_col;
        }
    }

    pub(super) fn delete_to_start(&mut self) {
        let end_offset = self
            .buf()
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        let start_offset = self.buf().line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
        if end_offset > start_offset {
            let content = self.buf().content();
            self.yank(content[start_offset..end_offset].to_string());
            self.buf().delete(start_offset, end_offset);
            self.cursor_col = 0;
        }
    }

    pub(super) fn delete_to_end(&mut self) {
        let start = self
            .buf()
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        let line_len = self.buf().line_len(self.cursor_line);
        let end = self
            .buf()
            .line_col_to_offset(self.cursor_line, line_len)
            .unwrap_or(start);
        if end > start {
            let content = self.buf().content();
            self.yank(content[start..end].to_string());
            self.buf().delete(start, end);
        }
        self.clamp_col();
    }

    pub(super) fn change_line(&mut self) {
        self.buf().begin_group();
        let start = self.buf().line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
        let line_len = self.buf().line_len(self.cursor_line);
        let end = self
            .buf()
            .line_col_to_offset(self.cursor_line, line_len)
            .unwrap_or(start);
        if end > start {
            self.buf().delete(start, end);
        }
        self.cursor_col = 0;
        self.mode = EditorMode::Insert;
    }
}
