//! Tests verifying widgets use the glyph system (no hardcoded chars).

use txv_core::prelude::*;

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
fn split_pane_draws_glyph_separator() {
    // Verify split pane draws whatever the current glyph set says
    let mut sp = SplitPane::new(
        SplitDirection::Horizontal,
        Box::new(Dummy::new()),
        Box::new(Dummy::new()),
    );
    sp.set_bounds(Rect::new(0, 0, 20, 10));

    sp.draw();

    let cell = sp.buffer().cell(10, 0);
    let g = glyphs();
    assert_eq!(cell.ch, g.ui.separator_v, "separator should match active glyph set");
}

#[test]
fn ascii_glyph_set_is_all_ascii() {
    let g = GlyphSet::ascii();
    assert_eq!(g.ui.separator_v, '|');
    assert_eq!(g.ui.separator_h, '-');
    assert_eq!(g.box_drawing.tl, '+');
    assert_eq!(g.box_drawing.h_heavy, '=');
    assert_eq!(g.tree.collapsed, "> ");
    assert_eq!(g.progress.empty, '.');
}

#[test]
fn unicode_glyph_set_has_box_drawing() {
    let g = GlyphSet::unicode();
    assert_eq!(g.box_drawing.h, '─');
    assert_eq!(g.box_drawing.v, '│');
    assert_eq!(g.box_drawing.tl, '┌');
    assert_eq!(g.box_drawing.h_heavy, '═');
    assert_eq!(g.box_drawing.tl_heavy, '╔');
    assert_eq!(g.tree.expanded, "▼ ");
}

#[test]
fn unicode_extended_has_rounded_corners() {
    let g = GlyphSet::unicode_extended();
    assert_eq!(g.box_drawing.tl_round, '╭');
    assert_eq!(g.box_drawing.br_round, '╯');
    // Standard corners unchanged
    assert_eq!(g.box_drawing.tl, '┌');
}
