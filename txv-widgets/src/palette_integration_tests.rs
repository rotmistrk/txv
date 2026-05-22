//! Tests verifying widgets use palette colors (no hardcoded values).
//!
//! These tests share global palette state, so they run as a single test
//! to avoid race conditions from parallel execution.

use txv_core::prelude::*;

use crate::inline_edit::InlineEditor;
use crate::split_pane::{SplitDirection, SplitPane};

struct Dummy {
    state: ViewState,
}
impl Dummy {
    fn new() -> Self {
        Self {
            state: ViewState::default(),
        }
    }
}
impl View for Dummy {
    delegate_view_state!(state);
    fn draw(&mut self) {}
    fn handle(&mut self, _: &Event) -> HandleResult {
        HandleResult::Ignored
    }
}

#[test]
fn palette_integration() {
    // --- inline_edit_selection_uses_palette ---
    set_palette(Palette::default());
    let pal = palette();
    let expected_bg = pal.interactive.edit_selection.bg.unwrap();

    let ed = InlineEditor::new_selected(0, "hello");
    let mut surface = Surface::new(20, 1);
    let style = Style::default();
    ed.draw(&mut surface, 0, 0, 20, style);

    let cell = surface.cell(1, 0);
    assert_eq!(cell.style.bg, expected_bg, "selection bg should come from palette");

    // --- split_pane_separator_uses_palette_dim ---
    set_palette(Palette::default());
    let pal = palette();
    let expected_fg = pal.base.dim.fg.unwrap();

    let mut sp = SplitPane::new(
        SplitDirection::Horizontal,
        Box::new(Dummy::new()),
        Box::new(Dummy::new()),
    );
    sp.set_bounds(Rect::new(0, 0, 20, 10));
    sp.draw();

    let cell = sp.buffer().cell(10, 0);
    assert_eq!(cell.style.fg, expected_fg, "separator should use palette dim color");

    // --- palette_change_affects_widget_rendering ---
    let custom_bg = Color::Ansi(5);
    let mut p = Palette::default();
    p.interactive.edit_selection = PaletteStyle::bg(custom_bg);
    set_palette(p);

    let ed = InlineEditor::new_selected(0, "test");
    let mut surface = Surface::new(20, 1);
    ed.draw(&mut surface, 0, 0, 20, Style::default());

    let cell = surface.cell(1, 0);
    assert_eq!(cell.style.bg, custom_bg, "widget should reflect updated palette");

    // Restore default
    set_palette(Palette::default());
}
