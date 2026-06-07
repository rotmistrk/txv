//! Tests verifying widgets use palette colors (no hardcoded values).

use std::sync::Arc;

use txv_core::prelude::*;

use crate::input_line::InputLine;
use crate::palette_test_helpers::Dummy;
use crate::split_pane::{SplitDirection, SplitPane};

#[test]
fn palette_integration() {
    use txv_core::palette::dark::DarkPalette;

    // --- input_line_selection_uses_palette ---
    set_palette(Arc::new(DarkPalette));
    let pal = palette();
    let expected_bg = pal.style(StyleId::EditSelection).bg();

    let mut il = InputLine::new();
    il.set_text("hello");
    il.select_all();
    il.set_bounds(Rect::new(0, 0, 20, 1));
    il.draw();

    let cell = il.buffer().cell(1, 0);
    assert_eq!(cell.style().bg(), expected_bg, "selection bg should come from palette");

    // --- split_pane_separator_uses_palette_dim ---
    let pal = palette();
    let expected_fg = pal.style(StyleId::Dim).fg();

    let mut sp = SplitPane::new(
        SplitDirection::Horizontal,
        Box::new(Dummy::new()),
        Box::new(Dummy::new()),
    );
    sp.set_bounds(Rect::new(0, 0, 20, 10));
    sp.draw();

    let cell = sp.buffer().cell(10, 0);
    assert_eq!(cell.style().fg(), expected_fg, "separator should use palette dim color");
}

#[test]
fn palette_change_affects_widget_rendering() {
    use txv_core::palette::dark::DarkPalette;

    use crate::palette_custom_tests::CustomPalette;

    set_palette(Arc::new(CustomPalette));

    let mut il = InputLine::new();
    il.set_text("test");
    il.select_all();
    il.set_bounds(Rect::new(0, 0, 20, 1));
    il.draw();

    let cell = il.buffer().cell(1, 0);
    assert_eq!(
        cell.style().bg(),
        Color::Ansi(5),
        "widget should reflect updated palette"
    );

    // Restore
    set_palette(Arc::new(DarkPalette));
}
