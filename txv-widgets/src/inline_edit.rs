//! InlineEditor — generic inline text editing for row-based widgets.

use txv_core::prelude::*;

/// Result of handling a key in the inline editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineEditResult {
    /// Key consumed, editing continues.
    Continue,
    /// User pressed Enter — commit the buffer.
    Commit(String),
    /// User pressed Escape — cancel editing.
    Cancel,
}

/// Delegate trait for inline editing behavior.
pub trait InlineEditDelegate: Send + 'static {
    /// Can the item at this visible row be edited?
    fn can_edit(&self, row: usize) -> bool;
    /// Validate in-progress text. None = valid, Some(msg) = error.
    fn validate(&self, row: usize, text: &str) -> Option<String>;
    /// Tab-completion candidates. Empty = no completions.
    fn complete(&self, _row: usize, _text: &str) -> Vec<String> {
        vec![]
    }
    /// Commit the edit. Called on Enter when validate returns None.
    fn commit(&mut self, row: usize, text: String);
}

/// Inline single-line editor embedded in a row.
pub struct InlineEditor {
    pub row: usize,
    pub buffer: String,
    pub cursor: usize,
    /// Selection anchor (byte offset). When Some, selection is anchor..cursor or cursor..anchor.
    pub anchor: Option<usize>,
    /// Horizontal scroll offset (char index) for long text.
    scroll_offset: usize,
}

impl InlineEditor {
    pub fn new(row: usize, initial_text: &str) -> Self {
        let cursor = initial_text.len();
        Self {
            row,
            buffer: initial_text.to_owned(),
            cursor,
            anchor: None,
            scroll_offset: 0,
        }
    }

    /// Create with entire text selected (anchor=0, cursor=end).
    pub fn new_selected(row: usize, initial_text: &str) -> Self {
        Self {
            row,
            buffer: initial_text.to_owned(),
            cursor: initial_text.len(),
            anchor: Some(0),
            scroll_offset: 0,
        }
    }

    /// Returns (start, end) byte offsets of selection, or None.
    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.anchor.map(|a| {
            if a <= self.cursor {
                (a, self.cursor)
            } else {
                (self.cursor, a)
            }
        })
    }

    fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.selection_range() {
            if start != end {
                self.buffer.drain(start..end);
                self.cursor = start;
                self.anchor = None;
                return true;
            }
            self.anchor = None;
        }
        false
    }

    /// Handle a key event. Returns the editing result.
    pub fn handle_key(&mut self, key: &KeyEvent) -> InlineEditResult {
        let shift = key.modifiers.shift;
        match key.code {
            KeyCode::Enter => InlineEditResult::Commit(self.buffer.clone()),
            KeyCode::Tab => InlineEditResult::Commit(self.buffer.clone()),
            KeyCode::Esc => InlineEditResult::Cancel,
            KeyCode::Char(ch) => {
                self.delete_selection();
                self.insert_char(ch);
                InlineEditResult::Continue
            }
            KeyCode::Backspace => {
                if !self.delete_selection() {
                    self.delete_before();
                }
                InlineEditResult::Continue
            }
            KeyCode::Delete => {
                if !self.delete_selection() {
                    self.delete_at();
                }
                InlineEditResult::Continue
            }
            KeyCode::Left => {
                if shift {
                    if self.anchor.is_none() {
                        self.anchor = Some(self.cursor);
                    }
                } else {
                    self.anchor = None;
                }
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                InlineEditResult::Continue
            }
            KeyCode::Right => {
                if shift {
                    if self.anchor.is_none() {
                        self.anchor = Some(self.cursor);
                    }
                } else {
                    self.anchor = None;
                }
                if self.cursor < self.buffer.len() {
                    self.cursor += 1;
                }
                InlineEditResult::Continue
            }
            KeyCode::Home => {
                if shift {
                    if self.anchor.is_none() {
                        self.anchor = Some(self.cursor);
                    }
                } else {
                    self.anchor = None;
                }
                self.cursor = 0;
                InlineEditResult::Continue
            }
            KeyCode::End => {
                if shift {
                    if self.anchor.is_none() {
                        self.anchor = Some(self.cursor);
                    }
                } else {
                    self.anchor = None;
                }
                self.cursor = self.buffer.len();
                InlineEditResult::Continue
            }
            _ => InlineEditResult::Continue,
        }
    }

    /// Draw the editor at the given position on the surface.
    pub fn draw(&mut self, surface: &mut Surface, x: u16, y: u16, width: u16, style: Style) {
        let w = width as usize;
        // Convert byte cursor to char index for scroll math
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
        let sel_bg = pal.style(StyleId::EditSelection).bg;
        let sel_style = Style {
            bg: if sel_bg != Color::Reset {
                sel_bg
            } else {
                style.bg
            },
            ..style
        };
        let cursor_style = Style {
            fg: style.bg,
            bg: style.fg,
            ..style
        };
        let sel = self.selection_range();
        surface.hline(x, y, width, ' ', style);
        // Render chars starting from scroll_offset
        let mut byte_pos = 0;
        for (ci, ch) in self.buffer.chars().enumerate() {
            if ci >= self.scroll_offset + w {
                break;
            }
            if ci >= self.scroll_offset {
                let vi = ci - self.scroll_offset;
                let st = if byte_pos == self.cursor {
                    cursor_style
                } else if sel.is_some_and(|(s, e)| byte_pos >= s && byte_pos < e) {
                    sel_style
                } else {
                    style
                };
                surface.put(x + vi as u16, y, ch, st);
            }
            byte_pos += ch.len_utf8();
        }
        // Cursor at end of text
        let visible_cursor = char_cursor.saturating_sub(self.scroll_offset);
        if self.cursor >= self.buffer.len() && visible_cursor < w {
            surface.put(x + visible_cursor as u16, y, ' ', cursor_style);
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

    fn insert_char(&mut self, ch: char) {
        self.buffer.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
    }

    fn delete_before(&mut self) {
        if self.cursor > 0 {
            let prev = self.buffer[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.buffer.drain(prev..self.cursor);
            self.cursor = prev;
        }
    }

    fn delete_at(&mut self) {
        if self.cursor < self.buffer.len() {
            let next = self.buffer[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.buffer.len());
            self.buffer.drain(self.cursor..next);
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
#[path = "inline_edit_tests.rs"]
mod tests;
