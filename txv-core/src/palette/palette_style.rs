//! PaletteStyle — a single palette entry for building implementations.

use crate::cell::{Attrs, Color, Style};

/// A single palette entry for building implementations.
#[derive(Clone, Debug, Default)]
pub struct PaletteStyle {
    fg: Option<Color>,
    bg: Option<Color>,
    bold: bool,
    italic: bool,
    underline: bool,
    dim: bool,
}

impl PaletteStyle {
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
            dim: false,
        }
    }
    pub const fn fg(color: Color) -> Self {
        Self {
            fg: Some(color),
            bg: None,
            bold: false,
            italic: false,
            underline: false,
            dim: false,
        }
    }
    pub const fn bg(color: Color) -> Self {
        Self {
            fg: None,
            bg: Some(color),
            bold: false,
            italic: false,
            underline: false,
            dim: false,
        }
    }
    pub const fn colors(fg: Color, bg: Color) -> Self {
        Self {
            fg: Some(fg),
            bg: Some(bg),
            bold: false,
            italic: false,
            underline: false,
            dim: false,
        }
    }
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
    pub fn dim(mut self) -> Self {
        self.dim = true;
        self
    }
    pub fn to_style(&self) -> Style {
        Style {
            fg: self.fg.unwrap_or(Color::Reset),
            bg: self.bg.unwrap_or(Color::Reset),
            attrs: Attrs {
                bold: self.bold,
                italic: self.italic,
                underline: self.underline,
                dim: self.dim,
            },
        }
    }
}
