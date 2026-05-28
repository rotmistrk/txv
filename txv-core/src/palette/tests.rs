//! Palette integration tests — verify defaults and trait access.

use std::sync::Arc;

use crate::cell::Color;
use crate::palette::dark::DarkPalette;
use crate::palette::style_id::StyleId;
use crate::palette::{palette, set_palette, Palette, PaletteStyle};

#[test]
fn dark_palette_has_expected_roles() {
    let p = palette();
    assert_eq!(p.style(StyleId::Dim).fg, Color::Ansi(8));
    assert_eq!(p.style(StyleId::TreeDir).fg, Color::Ansi(14));
    assert_eq!(p.style(StyleId::CursorFocused).bg, Color::Ansi(4));
    assert_eq!(p.style(StyleId::CursorUnfocused).bg, Color::Ansi(8));
    assert_eq!(p.style(StyleId::EditSelection).bg, Color::Ansi(2));
    assert_eq!(p.style(StyleId::PopupBorder).fg, Color::Ansi(6));
    assert_eq!(p.style(StyleId::PopupBackground).fg, Color::Ansi(15));
    assert_eq!(p.style(StyleId::PopupSelected).bg, Color::Ansi(4));
}

#[test]
fn set_palette_round_trip() {
    set_palette(Arc::new(DarkPalette));
    let got = palette();
    assert_eq!(got.style(StyleId::StateError).fg, Color::Ansi(9));
}

#[test]
fn palette_style_to_style_resolves_correctly() {
    let ps = PaletteStyle::colors(Color::Ansi(15), Color::Ansi(4));
    let s = ps.to_style();
    assert_eq!(s.fg, Color::Ansi(15));
    assert_eq!(s.bg, Color::Ansi(4));
}
