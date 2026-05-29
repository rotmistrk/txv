//! Tests verifying widgets use palette colors (no hardcoded values).

use std::sync::Arc;

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
    use txv_core::palette::dark::DarkPalette;

    // --- inline_edit_selection_uses_palette ---
    set_palette(Arc::new(DarkPalette));
    let pal = palette();
    let expected_bg = pal.style(StyleId::EditSelection).bg;

    let mut ed = InlineEditor::new_selected(0, "hello");
    let mut surface = Surface::new(20, 1);
    let style = Style::default();
    ed.draw(&mut surface, 0, 0, 20, style);

    let cell = surface.cell(1, 0);
    assert_eq!(cell.style.bg, expected_bg, "selection bg should come from palette");

    // --- split_pane_separator_uses_palette_dim ---
    let pal = palette();
    let expected_fg = pal.style(StyleId::Dim).fg;

    let mut sp = SplitPane::new(
        SplitDirection::Horizontal,
        Box::new(Dummy::new()),
        Box::new(Dummy::new()),
    );
    sp.set_bounds(Rect::new(0, 0, 20, 10));
    sp.draw();

    let cell = sp.buffer().cell(10, 0);
    assert_eq!(cell.style.fg, expected_fg, "separator should use palette dim color");
}

#[test]
fn palette_change_affects_widget_rendering() {
    use txv_core::palette::dark::DarkPalette;
    use txv_core::palette::style_id::StyleId;

    // Set a custom palette with different selection color
    struct CustomPalette;
    impl txv_core::palette::Palette for CustomPalette {
        fn style(&self, id: StyleId) -> Style {
            if id == StyleId::EditSelection {
                Style {
                    bg: Color::Ansi(5),
                    ..Style::default()
                }
            } else {
                DarkPalette.style(id)
            }
        }
    }
    set_palette(Arc::new(CustomPalette));

    let mut ed = InlineEditor::new_selected(0, "test");
    let mut surface = Surface::new(20, 1);
    ed.draw(&mut surface, 0, 0, 20, Style::default());

    let cell = surface.cell(1, 0);
    assert_eq!(cell.style.bg, Color::Ansi(5), "widget should reflect updated palette");

    // Restore
    set_palette(Arc::new(DarkPalette));
}
