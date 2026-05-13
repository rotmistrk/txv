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
}

impl InlineEditor {
    pub fn new(row: usize, initial_text: &str) -> Self {
        let cursor = initial_text.len();
        Self {
            row,
            buffer: initial_text.to_owned(),
            cursor,
            anchor: None,
        }
    }

    /// Create with entire text selected (anchor=0, cursor=end).
    pub fn new_selected(row: usize, initial_text: &str) -> Self {
        Self {
            row,
            buffer: initial_text.to_owned(),
            cursor: initial_text.len(),
            anchor: Some(0),
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
    pub fn draw(&self, surface: &mut Surface, x: u16, y: u16, width: u16, style: Style) {
        let sel_style = Style {
            bg: Color::Ansi(2),
            ..style
        };
        let cursor_style = Style {
            fg: style.bg,
            bg: style.fg,
            ..style
        };
        let sel = self.selection_range();
        surface.hline(x, y, width, ' ', style);
        let w = width as usize;
        for (i, ch) in self.buffer.chars().enumerate() {
            if i >= w {
                break;
            }
            let st = if i == self.cursor {
                cursor_style
            } else if sel.is_some_and(|(s, e)| i >= s && i < e) {
                sel_style
            } else {
                style
            };
            surface.put(x + i as u16, y, ch, st);
        }
        // Draw cursor at end if past last char
        if self.cursor >= self.buffer.len() && (self.cursor as u16) < width {
            surface.put(x + self.cursor as u16, y, ' ', cursor_style);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyMod::default(),
        }
    }

    #[test]
    fn insert_and_commit() {
        let mut ed = InlineEditor::new(0, "");
        assert_eq!(ed.handle_key(&key(KeyCode::Char('h'))), InlineEditResult::Continue);
        assert_eq!(ed.handle_key(&key(KeyCode::Char('i'))), InlineEditResult::Continue);
        assert_eq!(ed.buffer, "hi");
        assert_eq!(ed.cursor, 2);
        assert_eq!(
            ed.handle_key(&key(KeyCode::Enter)),
            InlineEditResult::Commit("hi".to_owned())
        );
    }

    #[test]
    fn cancel() {
        let mut ed = InlineEditor::new(0, "text");
        assert_eq!(ed.handle_key(&key(KeyCode::Esc)), InlineEditResult::Cancel);
    }

    #[test]
    fn backspace_and_delete() {
        let mut ed = InlineEditor::new(0, "abc");
        // cursor at end (3)
        ed.handle_key(&key(KeyCode::Backspace));
        assert_eq!(ed.buffer, "ab");
        assert_eq!(ed.cursor, 2);
        ed.handle_key(&key(KeyCode::Home));
        ed.handle_key(&key(KeyCode::Delete));
        assert_eq!(ed.buffer, "b");
        assert_eq!(ed.cursor, 0);
    }

    #[test]
    fn navigation() {
        let mut ed = InlineEditor::new(0, "hello");
        ed.handle_key(&key(KeyCode::Home));
        assert_eq!(ed.cursor, 0);
        ed.handle_key(&key(KeyCode::Right));
        assert_eq!(ed.cursor, 1);
        ed.handle_key(&key(KeyCode::End));
        assert_eq!(ed.cursor, 5);
        ed.handle_key(&key(KeyCode::Left));
        assert_eq!(ed.cursor, 4);
    }

    #[test]
    fn tab_completion() {
        let mut ed = InlineEditor::new(0, "");
        let candidates = vec!["alpha".to_owned(), "beta".to_owned(), "gamma".to_owned()];
        ed.apply_completion(&candidates, 1);
        assert_eq!(ed.buffer, "alpha");
        ed.apply_completion(&candidates, 1);
        assert_eq!(ed.buffer, "beta");
        ed.apply_completion(&candidates, 1);
        assert_eq!(ed.buffer, "gamma");
        ed.apply_completion(&candidates, 1);
        assert_eq!(ed.buffer, "alpha");
    }

    fn shift_key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyMod {
                shift: true,
                ..KeyMod::default()
            },
        }
    }

    #[test]
    fn new_selected_selects_all() {
        let ed = InlineEditor::new_selected(0, "hello");
        assert_eq!(ed.anchor, Some(0));
        assert_eq!(ed.cursor, 5);
        assert_eq!(ed.selection_range(), Some((0, 5)));
    }

    #[test]
    fn type_replaces_selection() {
        let mut ed = InlineEditor::new_selected(0, "old");
        ed.handle_key(&key(KeyCode::Char('n')));
        assert_eq!(ed.buffer, "n");
        assert_eq!(ed.cursor, 1);
        assert_eq!(ed.anchor, None);
    }

    #[test]
    fn shift_arrow_extends_selection() {
        let mut ed = InlineEditor::new(0, "abcde");
        ed.handle_key(&key(KeyCode::Home));
        ed.handle_key(&shift_key(KeyCode::Right));
        ed.handle_key(&shift_key(KeyCode::Right));
        assert_eq!(ed.selection_range(), Some((0, 2)));
        // Nav without shift clears selection
        ed.handle_key(&key(KeyCode::Right));
        assert_eq!(ed.anchor, None);
    }

    #[test]
    fn backspace_deletes_selection() {
        let mut ed = InlineEditor::new_selected(0, "hello");
        ed.handle_key(&key(KeyCode::Backspace));
        assert_eq!(ed.buffer, "");
        assert_eq!(ed.cursor, 0);
    }

    #[test]
    fn tab_commits() {
        let mut ed = InlineEditor::new(0, "text");
        assert_eq!(
            ed.handle_key(&key(KeyCode::Tab)),
            InlineEditResult::Commit("text".to_owned())
        );
    }

    #[test]
    fn shift_home_selects_to_start() {
        let mut ed = InlineEditor::new(0, "hello");
        // cursor at end (5)
        ed.handle_key(&shift_key(KeyCode::Home));
        assert_eq!(ed.selection_range(), Some((0, 5)));
        assert_eq!(ed.cursor, 0);
        assert_eq!(ed.anchor, Some(5));
    }

    #[test]
    fn shift_end_selects_to_end() {
        let mut ed = InlineEditor::new(0, "hello");
        ed.handle_key(&key(KeyCode::Home));
        ed.handle_key(&shift_key(KeyCode::End));
        assert_eq!(ed.selection_range(), Some((0, 5)));
        assert_eq!(ed.cursor, 5);
        assert_eq!(ed.anchor, Some(0));
    }

    #[test]
    fn delete_key_removes_selection() {
        let mut ed = InlineEditor::new(0, "abcde");
        ed.handle_key(&key(KeyCode::Home));
        ed.handle_key(&shift_key(KeyCode::Right));
        ed.handle_key(&shift_key(KeyCode::Right));
        ed.handle_key(&shift_key(KeyCode::Right));
        // selection is 0..3
        ed.handle_key(&key(KeyCode::Delete));
        assert_eq!(ed.buffer, "de");
        assert_eq!(ed.cursor, 0);
    }

    #[test]
    fn type_mid_selection_replaces() {
        let mut ed = InlineEditor::new(0, "abcde");
        ed.cursor = 1;
        ed.anchor = Some(4); // select "bcd"
        ed.handle_key(&key(KeyCode::Char('X')));
        assert_eq!(ed.buffer, "aXe");
        assert_eq!(ed.cursor, 2);
    }

    #[test]
    fn selection_range_none_without_anchor() {
        let ed = InlineEditor::new(0, "text");
        assert_eq!(ed.selection_range(), None);
    }

    #[test]
    fn nav_after_selection_clears_anchor() {
        let mut ed = InlineEditor::new_selected(0, "abc");
        ed.handle_key(&key(KeyCode::Left));
        assert_eq!(ed.anchor, None);
        // cursor moved
        assert_eq!(ed.cursor, 2);
    }
}
