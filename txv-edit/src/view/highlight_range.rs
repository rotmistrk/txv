//! HighlightRange + CursorRender — styling types for EditorViewDelegate.

use txv_core::prelude::Style;

/// A highlighted range on a line — merged onto existing cell style.
pub struct HighlightRange {
    col_start: usize,
    col_end: usize,
    style: Style,
}

impl HighlightRange {
    pub fn new(col_start: usize, col_end: usize, style: Style) -> Self {
        Self {
            col_start,
            col_end,
            style,
        }
    }

    pub fn col_start(&self) -> usize {
        self.col_start
    }

    pub fn col_end(&self) -> usize {
        self.col_end
    }

    pub fn style(&self) -> Style {
        self.style
    }
}

/// Cursor rendering mode.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CursorRender {
    Hardware,
    Software(Style),
    None,
}
