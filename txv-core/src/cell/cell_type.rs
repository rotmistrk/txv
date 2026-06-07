//! Cell — the atomic drawing unit.

use super::Style;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Cell {
    pub(crate) ch: char,
    pub(crate) style: Style,
    pub(crate) width: u8,
}

impl Cell {
    pub fn new(ch: char, style: Style, width: u8) -> Self {
        Self { ch, style, width }
    }

    pub fn ch(&self) -> char {
        self.ch
    }
    pub fn style(&self) -> Style {
        self.style
    }
    pub fn width(&self) -> u8 {
        self.width
    }
    pub fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
    pub fn set_ch(&mut self, ch: char) {
        self.ch = ch;
    }
    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            style: Style::default(),
            width: 1,
        }
    }
}
