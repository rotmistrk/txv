//! Tests verifying widgets use palette colors (no hardcoded values).

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
    fn draw(&self, _: &mut Surface) {}
    fn handle(&mut self, _: &Event, _: &mut EventQueue) -> HandleResult {
        HandleResult::Ignored
    }
}

#[test]
fn inline_edit_selection_uses_palette() {
    set_palette(Palette::default());
    let pal = palette();
    let expected_bg = pal.interactive.edit_selection.bg.unwrap();

    let ed = InlineEditor::new_selected(0, "hello");
    let mut surface = Surface::new(20, 1);
    let style = Style::default();
    ed.draw(&mut surface, 0, 0, 20, style);

    // Selected chars (0..4) should have palette selection bg
    let cell = surface.cell(1, 0);
    assert_eq!(cell.style.bg, expected_bg, "selection bg should come from palette");
}

#[test]
fn split_pane_separator_uses_palette_dim() {
    set_palette(Palette::default());
    let pal = palette();
    let expected_fg = pal.base.dim.fg.unwrap();

    let mut sp = SplitPane::new(
        SplitDirection::Horizontal,
        Box::new(Dummy::new()),
        Box::new(Dummy::new()),
    );
    sp.set_bounds(Rect::new(0, 0, 20, 10));

    let mut surface = Surface::new(20, 10);
    sp.draw(&mut surface);

    // Separator at x=10 should use dim fg
    let cell = surface.cell(10, 0);
    assert_eq!(cell.style.fg, expected_fg, "separator should use palette dim color");
}

#[test]
fn palette_change_affects_widget_rendering() {
    // Set a custom palette with different selection color
    let mut p = Palette::default();
    p.interactive.edit_selection = PaletteStyle::bg(Color::Ansi(5));
    set_palette(p);

    let ed = InlineEditor::new_selected(0, "test");
    let mut surface = Surface::new(20, 1);
    ed.draw(&mut surface, 0, 0, 20, Style::default());

    let cell = surface.cell(1, 0);
    assert_eq!(cell.style.bg, Color::Ansi(5), "widget should reflect updated palette");

    // Restore
    set_palette(Palette::default());
}
