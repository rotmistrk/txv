//! Style — foreground, background, and attributes.

use super::{Attrs, Color};
use std::mem;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Style {
    pub(crate) fg: Color,
    pub(crate) bg: Color,
    pub(crate) attrs: Attrs,
}

impl Style {
    pub fn new(fg: Color, bg: Color) -> Self {
        Self {
            fg,
            bg,
            attrs: Attrs::default(),
        }
    }

    pub fn fg(&self) -> Color {
        self.fg
    }
    pub fn bg(&self) -> Color {
        self.bg
    }
    pub fn attrs(&self) -> Attrs {
        self.attrs
    }

    pub fn with_attrs(mut self, attrs: Attrs) -> Self {
        self.attrs = attrs;
        self
    }

    pub fn set_fg(&mut self, fg: Color) {
        self.fg = fg;
    }
    pub fn set_bg(&mut self, bg: Color) {
        self.bg = bg;
    }
    pub fn set_attrs(&mut self, attrs: Attrs) {
        self.attrs = attrs;
    }
    pub fn attrs_mut(&mut self) -> &mut Attrs {
        &mut self.attrs
    }

    pub fn swap_fg_bg(&mut self) {
        mem::swap(&mut self.fg, &mut self.bg);
    }

    pub fn with_fg(mut self, fg: Color) -> Self {
        self.fg = fg;
        self
    }

    pub fn with_bg(mut self, bg: Color) -> Self {
        self.bg = bg;
        self
    }
}
