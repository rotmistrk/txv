//! PieceTable — efficient text buffer for editing.
//!
//! Supports O(log n) insert/delete, line indexing, and undo/redo.
//! This is a placeholder that will be populated by moving code from kairn.

/// Piece table text buffer.
pub struct PieceTable {
    text: String,
}

impl PieceTable {
    pub fn new(text: &str) -> Self {
        Self { text: text.to_string() }
    }

    pub fn line_count(&self) -> usize {
        self.text.lines().count().max(1)
    }

    pub fn line(&self, idx: usize) -> Option<String> {
        self.text.lines().nth(idx).map(String::from)
    }

    pub fn full_text(&self) -> &str {
        &self.text
    }
}
