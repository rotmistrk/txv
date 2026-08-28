//! InputLine selection and text manipulation methods.

use txv_core::prelude::Rect;

use super::InputLine;

impl InputLine {
    pub(crate) fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    pub(crate) fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection.map(|anchor| {
            let lo = anchor.min(self.cursor);
            let hi = anchor.max(self.cursor);
            (lo, hi)
        })
    }

    /// Get the currently selected text, if any.
    pub fn selected_text(&self) -> Option<String> {
        let (lo, hi) = self.selection_range()?;
        let byte_lo = self.char_to_byte(lo);
        let byte_hi = self.char_to_byte(hi);
        Some(self.text[byte_lo..byte_hi].to_string())
    }

    /// Insert text at cursor, replacing selection if active.
    pub fn insert_text(&mut self, text: &str) {
        if self.selection.is_some() {
            self.delete_selection();
        }
        let byte_pos = self.char_to_byte(self.cursor);
        self.text.insert_str(byte_pos, text);
        self.cursor += text.chars().count();
        self.update_width();
        self.state.mark_dirty();
    }

    pub(crate) fn delete_selection(&mut self) {
        if let Some((lo, hi)) = self.selection_range() {
            let byte_lo = self.char_to_byte(lo);
            let byte_hi = self.char_to_byte(hi);
            self.text.drain(byte_lo..byte_hi);
            self.cursor = lo;
            self.selection = None;
            self.update_width();
        }
    }

    /// Convert char index to byte offset.
    pub(crate) fn char_to_byte(&self, char_idx: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_idx)
            .map(|(b, _)| b)
            .unwrap_or(self.text.len())
    }

    /// Auto-resize bounds to fit text (standalone mode only).
    pub(crate) fn update_width(&mut self) {
        self.state.mark_dirty();
        if self.constrained {
            return;
        }
        let w = (self.char_count() as u16).saturating_add(2).max(10);
        let b = self.state.bounds();
        if b.w() != w {
            self.state.set_bounds(Rect::new(b.x(), b.y(), w, 1));
        }
    }

    pub(crate) fn handle_nav(&mut self, shift: bool, new_cursor: usize) {
        if shift {
            if self.selection.is_none() {
                self.selection = Some(self.cursor);
            }
        } else {
            self.selection = None;
        }
        self.cursor = new_cursor;
        self.state.mark_dirty();
    }

    pub(crate) fn visible_start(&self, width: usize) -> usize {
        if width == 0 {
            return 0;
        }
        let total = self.char_count();
        let mut start = if self.cursor >= width {
            self.cursor - width + 1
        } else {
            0
        };
        let cursor_pos = self.cursor - start;
        if cursor_pos == width - 1 && start + width < total {
            start += 1;
        }
        if start > 0 && self.cursor == start {
            start -= 1;
        }
        start
    }
}
