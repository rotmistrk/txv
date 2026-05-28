//! Palette integration tests — verify defaults and trait access.

use std::sync::Arc;

use crate::cell::Color;
use crate::palette::dark::DarkPalette;
use crate::palette::{palette, set_palette, PaletteStyle};

#[test]
fn dark_palette_has_expected_roles() {
    let p = palette();
    assert_eq!(p.base().dim().fg, Color::Ansi(8));
    assert_eq!(p.base().tree_dir().fg, Color::Ansi(14));
    assert_eq!(p.interactive().cursor_focused().bg, Color::Ansi(4));
    assert_eq!(p.interactive().cursor_unfocused().bg, Color::Ansi(8));
    assert_eq!(p.interactive().edit_selection().bg, Color::Ansi(2));
    assert_eq!(p.popup().border().fg, Color::Ansi(6));
    assert_eq!(p.popup().background().fg, Color::Ansi(15));
    assert_eq!(p.popup().selected().bg, Color::Ansi(4));
}

#[test]
fn set_palette_round_trip() {
    set_palette(Arc::new(DarkPalette));
    let got = palette();
    assert_eq!(got.state().error().fg, Color::Ansi(9));
}

#[test]
fn palette_style_to_style_resolves_correctly() {
    let ps = PaletteStyle::colors(Color::Ansi(15), Color::Ansi(4));
    let s = ps.to_style();
    assert_eq!(s.fg, Color::Ansi(15));
    assert_eq!(s.bg, Color::Ansi(4));
}
