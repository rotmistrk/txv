//! Palette integration tests — verify defaults and set/get round-trip.

use crate::cell::Color;
use crate::palette::{palette, set_palette, Palette, PaletteStyle};

#[test]
fn default_palette_has_expected_roles() {
    let p = Palette::default();
    // Base
    assert_eq!(p.base.dim.fg, Some(Color::Ansi(8)));
    assert_eq!(p.base.tree_dir.fg, Some(Color::Ansi(14)));
    // Interactive
    assert_eq!(p.interactive.cursor_focused.bg, Some(Color::Ansi(4)));
    assert_eq!(p.interactive.cursor_unfocused.bg, Some(Color::Ansi(8)));
    assert_eq!(p.interactive.edit_selection.bg, Some(Color::Ansi(2)));
    // Popup
    assert_eq!(p.popup.border.fg, Some(Color::Ansi(6)));
    assert_eq!(p.popup.background.fg, Some(Color::Ansi(15)));
    assert_eq!(p.popup.selected.bg, Some(Color::Ansi(4)));
}

#[test]
fn set_palette_round_trip() {
    let mut p = Palette::default();
    p.base.tree_dir = PaletteStyle::fg(Color::Ansi(10));
    set_palette(p);
    let got = palette();
    assert_eq!(got.base.tree_dir.fg, Some(Color::Ansi(10)));
    // Restore default
    set_palette(Palette::default());
}

#[test]
fn palette_style_to_style_resolves_correctly() {
    let ps = PaletteStyle::colors(Color::Ansi(15), Color::Ansi(4));
    let s = ps.to_style();
    assert_eq!(s.fg, Color::Ansi(15));
    assert_eq!(s.bg, Color::Ansi(4));
}
