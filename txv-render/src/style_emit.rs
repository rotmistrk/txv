//! Style and color emission helpers for crossterm backend.

use std::io::Write;

use crossterm::{
    queue,
    style::{self, Attribute, SetAttribute},
};
use txv_core::cell::{Attrs, Style};

use crate::color::{downgrade, ColorMode};

pub(crate) fn emit_attrs(out: &mut impl Write, attrs: Attrs) {
    if attrs.bold {
        queue!(out, SetAttribute(Attribute::Bold)).ok();
    }
    if attrs.dim {
        queue!(out, SetAttribute(Attribute::Dim)).ok();
    }
    if attrs.italic {
        queue!(out, SetAttribute(Attribute::Italic)).ok();
    }
    if attrs.underline {
        queue!(out, SetAttribute(Attribute::Underlined)).ok();
    }
    if attrs.reverse {
        queue!(out, SetAttribute(Attribute::Reverse)).ok();
    }
}

pub(crate) fn apply_color_mode(s: Style, mode: ColorMode) -> Style {
    Style {
        fg: downgrade(s.fg, mode),
        bg: downgrade(s.bg, mode),
        attrs: s.attrs,
    }
}

pub(crate) fn emit_style(out: &mut impl Write, s: &Style) {
    queue!(out, SetAttribute(Attribute::Reset)).ok();
    queue!(out, style::SetForegroundColor(to_crossterm_color(s.fg))).ok();
    queue!(out, style::SetBackgroundColor(to_crossterm_color(s.bg))).ok();
    emit_attrs(out, s.attrs);
}

pub(crate) fn to_crossterm_color(color: txv_core::cell::Color) -> style::Color {
    match color {
        txv_core::cell::Color::Reset => style::Color::Reset,
        txv_core::cell::Color::Ansi(n) => style::Color::AnsiValue(n),
        txv_core::cell::Color::Palette(n) => style::Color::AnsiValue(n),
        txv_core::cell::Color::Rgb(r, g, b) => style::Color::Rgb { r, g, b },
    }
}
