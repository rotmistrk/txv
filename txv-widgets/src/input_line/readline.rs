//! Readline-style editing operations for InputLine.

use super::InputLine;

impl InputLine {
    /// Kill from cursor to end of line. Returns killed text.
    pub(crate) fn kill_to_end(&mut self) -> String {
        let byte_pos = self.char_to_byte(self.cursor);
        let killed = self.text[byte_pos..].to_string();
        self.text.truncate(byte_pos);
        self.update_width();
        self.state.mark_dirty();
        killed
    }

    /// Kill from start to cursor. Returns killed text.
    pub(crate) fn kill_to_start(&mut self) -> String {
        let byte_pos = self.char_to_byte(self.cursor);
        let killed = self.text[..byte_pos].to_string();
        self.text = self.text[byte_pos..].to_string();
        self.cursor = 0;
        self.update_width();
        self.state.mark_dirty();
        killed
    }

    /// Kill word backward. Returns killed text.
    pub(crate) fn kill_word_back(&mut self) -> String {
        let start = self.word_backward();
        let byte_start = self.char_to_byte(start);
        let byte_end = self.char_to_byte(self.cursor);
        let killed = self.text[byte_start..byte_end].to_string();
        self.text.replace_range(byte_start..byte_end, "");
        self.cursor = start;
        self.update_width();
        self.state.mark_dirty();
        killed
    }

    /// Kill word forward. Returns killed text.
    pub(crate) fn kill_word_forward(&mut self) -> String {
        let end = self.word_forward();
        let byte_start = self.char_to_byte(self.cursor);
        let byte_end = self.char_to_byte(end);
        let killed = self.text[byte_start..byte_end].to_string();
        self.text.replace_range(byte_start..byte_end, "");
        self.update_width();
        self.state.mark_dirty();
        killed
    }

    /// Transpose characters before cursor.
    pub(crate) fn transpose_chars(&mut self) {
        if self.cursor < 2 {
            return;
        }
        let chars: Vec<char> = self.text.chars().collect();
        let i = self.cursor - 1;
        if i >= chars.len() {
            return;
        }
        let mut new_chars = chars;
        new_chars.swap(i - 1, i);
        self.text = new_chars.into_iter().collect();
        self.state.mark_dirty();
    }

    /// Find word boundary forward from cursor.
    pub(crate) fn word_forward(&self) -> usize {
        let chars: Vec<char> = self.text.chars().collect();
        let mut pos = self.cursor;
        // Skip non-word chars
        while pos < chars.len() && !chars[pos].is_alphanumeric() {
            pos += 1;
        }
        // Skip word chars
        while pos < chars.len() && chars[pos].is_alphanumeric() {
            pos += 1;
        }
        pos
    }

    /// Find word boundary backward from cursor.
    pub(crate) fn word_backward(&self) -> usize {
        let chars: Vec<char> = self.text.chars().collect();
        let mut pos = self.cursor;
        // Skip non-word chars
        while pos > 0 && !chars[pos - 1].is_alphanumeric() {
            pos -= 1;
        }
        // Skip word chars
        while pos > 0 && chars[pos - 1].is_alphanumeric() {
            pos -= 1;
        }
        pos
    }
}
