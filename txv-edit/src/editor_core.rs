//! EditorCore — buffer, cursor, selections, undo/redo, and motion primitives.
//!
//! This is the mode-agnostic editing engine. All text manipulation goes through here.
//! Keybinding layers (vi, emacs) call EditorCore methods to effect changes.

use crate::buffer::PieceTable;

/// Core editor state: buffer + cursor + selection + undo.
pub struct EditorCore {
    buf: PieceTable,
    cursor_line: usize,
    cursor_col: usize,
    /// Desired column for vertical movement (remembers column across shorter lines).
    desired_col: usize,
}

impl EditorCore {
    pub fn new() -> Self {
        Self {
            buf: PieceTable::new(),
            cursor_line: 0,
            cursor_col: 0,
            desired_col: 0,
        }
    }

    pub fn from_text(text: &str) -> Self {
        Self {
            buf: PieceTable::from_text(text),
            cursor_line: 0,
            cursor_col: 0,
            desired_col: 0,
        }
    }

    pub fn buf(&self) -> &PieceTable {
        &self.buf
    }

    pub fn buf_mut(&mut self) -> &mut PieceTable {
        &mut self.buf
    }

    pub fn cursor_line(&self) -> usize {
        self.cursor_line
    }

    pub fn cursor_col(&self) -> usize {
        self.cursor_col
    }

    pub fn set_cursor(&mut self, line: usize, col: usize) {
        self.cursor_line = line;
        self.cursor_col = col;
        self.desired_col = col;
    }
}

impl Default for EditorCore {
    fn default() -> Self {
        Self::new()
    }
}
