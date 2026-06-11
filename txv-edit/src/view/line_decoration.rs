//! LineDecoration — underline/squiggly/background for a line range.

use txv_core::prelude::Color;

/// A decoration on a line segment (underline, squiggly, background).
pub struct LineDecoration {
    col_start: usize,
    col_end: usize,
    style: DecorationStyle,
}

impl LineDecoration {
    pub fn new(col_start: usize, col_end: usize, style: DecorationStyle) -> Self {
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

    pub fn style(&self) -> &DecorationStyle {
        &self.style
    }
}

/// Visual style for a line decoration.
pub enum DecorationStyle {
    Underline(Color),
    Squiggly(Color),
    Background(Color),
}
