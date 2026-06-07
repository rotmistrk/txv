//! Style and color emission helpers for crossterm backend.

use std::io::Write;

use crossterm::{
    queue,
    style::{self, Attribute, Color as CtColor, SetAttribute},
};
use txv_core::cell::{Attrs, Color, Style};

use crate::color::{downgrade, ColorMode};

pub(crate) fn emit_attrs(out: &mut impl Write, attrs: Attrs) {
    if attrs.bold_val() {
        queue!(out, SetAttribute(Attribute::Bold)).ok();
    }
    if attrs.dim_val() {
        queue!(out, SetAttribute(Attribute::Dim)).ok();
    }
    if attrs.italic_val() {
        queue!(out, SetAttribute(Attribute::Italic)).ok();
    }
    if attrs.underline_val() {
        queue!(out, SetAttribute(Attribute::Underlined)).ok();
    }
}

pub(crate) fn apply_color_mode(s: Style, mode: ColorMode) -> Style {
    Style::new(downgrade(s.fg(), mode), downgrade(s.bg(), mode)).with_attrs(s.attrs())
}

pub(crate) fn emit_style(out: &mut impl Write, s: &Style) {
    queue!(out, SetAttribute(Attribute::Reset)).ok();
    queue!(out, style::SetForegroundColor(to_crossterm_color(s.fg()))).ok();
    queue!(out, style::SetBackgroundColor(to_crossterm_color(s.bg()))).ok();
    emit_attrs(out, s.attrs());
}

pub(crate) fn to_crossterm_color(color: Color) -> CtColor {
    match color {
        Color::Reset | Color::Transparent => CtColor::Reset,
        Color::Ansi(n) => CtColor::AnsiValue(n),
        Color::Palette(n) => CtColor::AnsiValue(n),
        Color::Rgb(r, g, b) => CtColor::Rgb { r, g, b },
    }
}
