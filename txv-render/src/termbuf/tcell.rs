//! TCell — a single terminal cell (character + style + width).

use txv_core::cell::Style;

/// A terminal cell: character, style, and display width.
#[derive(Clone)]
pub struct TCell {
    pub(crate) ch: char,
    pub(crate) style: Style,
    #[allow(dead_code)]
    pub(crate) width: u8,
}

impl TCell {
    pub fn ch(&self) -> char {
        self.ch
    }
    pub fn style(&self) -> Style {
        self.style
    }
}

impl Default for TCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            style: Style::default(),
            width: 1,
        }
    }
}
