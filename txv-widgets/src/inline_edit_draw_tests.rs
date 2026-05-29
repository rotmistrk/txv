use std::sync::Arc;

use super::*;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyMod::default(),
    }
}

#[test]
fn scroll_offset_adjusts_on_draw() {
    let mut ed = InlineEditor::new(0, "abcdefghij");
    let mut buf = Buffer::new(5, 1);
    ed.draw(&mut buf, 0, 0, 5, Style::default());
    assert_eq!(ed.scroll_offset, 6); // cursor(10) - width(5) + 1 = 6
}

#[test]
fn scroll_offset_follows_cursor_left() {
    let mut ed = InlineEditor::new(0, "abcdefghij");
    let mut buf = Buffer::new(5, 1);
    ed.draw(&mut buf, 0, 0, 5, Style::default());
    assert_eq!(ed.scroll_offset, 6);
    ed.handle_key(&key(KeyCode::Home));
    ed.draw(&mut buf, 0, 0, 5, Style::default());
    assert_eq!(ed.scroll_offset, 0);
}

#[test]
fn scroll_offset_zero_when_text_fits() {
    let mut ed = InlineEditor::new(0, "hi");
    let mut buf = Buffer::new(10, 1);
    ed.draw(&mut buf, 0, 0, 10, Style::default());
    assert_eq!(ed.scroll_offset, 0);
}

#[test]
fn overflow_indicator_shown_at_right_when_scrolled_to_start() {
    use txv_core::palette::{dark::DarkPalette, set_palette};
    set_palette(Arc::new(DarkPalette));
    let mut ed = InlineEditor::new(0, "abcdefghij");
    ed.handle_key(&key(KeyCode::Home));
    let mut buf = Buffer::new(5, 1);
    ed.draw(&mut buf, 0, 0, 5, Style::default());
    let cell = buf.cell(4, 0);
    assert_eq!(cell.ch, '…');
    let ov_fg = palette().style(StyleId::OverflowIndicator).fg;
    assert_eq!(cell.style.fg, ov_fg);
}

#[test]
fn overflow_indicator_shown_at_left_when_scrolled() {
    use txv_core::palette::{dark::DarkPalette, set_palette};
    set_palette(Arc::new(DarkPalette));
    let mut ed = InlineEditor::new(0, "abcdefghij");
    let mut buf = Buffer::new(5, 1);
    ed.draw(&mut buf, 0, 0, 5, Style::default());
    assert!(ed.scroll_offset > 0);
    let cell = buf.cell(0, 0);
    assert_eq!(cell.ch, '…');
    let ov_fg = palette().style(StyleId::OverflowIndicator).fg;
    assert_eq!(cell.style.fg, ov_fg);
}

#[test]
fn no_overflow_indicator_when_text_fits() {
    let mut ed = InlineEditor::new(0, "hi");
    let mut buf = Buffer::new(10, 1);
    ed.draw(&mut buf, 0, 0, 10, Style::default());
    assert_ne!(buf.cell(0, 0).ch, '…');
    assert_ne!(buf.cell(9, 0).ch, '…');
}

#[test]
fn edit_overlay_style_uses_yellow_fg_reset_bg() {
    use txv_core::palette::{dark::DarkPalette, set_palette};
    set_palette(Arc::new(DarkPalette));
    let style = palette().style(StyleId::EditOverlay);
    assert_eq!(style.fg, Color::Ansi(3), "EditOverlay fg should be yellow");
    assert_eq!(style.bg, Color::Reset, "EditOverlay bg should be Reset (inherit)");
}

#[test]
fn draw_with_reset_bg_does_not_clear_underlying() {
    use txv_core::palette::{dark::DarkPalette, set_palette};
    set_palette(Arc::new(DarkPalette));
    let mut buf = Buffer::new(10, 1);
    // Pre-fill with a blue background
    let bg_style = Style {
        bg: Color::Ansi(4),
        ..Style::default()
    };
    buf.hline(0, 0, 10, ' ', bg_style);
    // Draw editor with Reset bg (EditOverlay style)
    let mut ed = InlineEditor::new(0, "hi");
    let edit_style = Style {
        fg: Color::Ansi(3),
        bg: Color::Reset,
        ..Style::default()
    };
    ed.draw(&mut buf, 0, 0, 10, edit_style);
    // Cells beyond text should retain blue bg
    let cell = buf.cell(5, 0);
    assert_eq!(
        cell.style.bg,
        Color::Ansi(4),
        "bg should be preserved when style.bg is Reset"
    );
}
