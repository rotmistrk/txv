//! Integration tests for hardware cursor propagation through the view hierarchy.

use txv_core::cursor::{CursorRequest, CursorShape};
use txv_core::prelude::*;

use crate::input_dialog::InputDialog;
use crate::input_line::InputLine;
use crate::split_pane::{SplitDirection, SplitPane};

/// A non-focusable dummy view (no cursor).
struct Dummy {
    state: ViewState,
}
impl Dummy {
    fn new() -> Self {
        Self {
            state: ViewState::new(ViewOptions {
                focusable: false,
                ..ViewOptions::default()
            }),
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
fn input_line_returns_cursor_when_focused() {
    let mut il = InputLine::new();
    il.set_bounds(Rect::new(0, 0, 40, 1));
    il.select();
    il.set_text("hello");

    let req = il.cursor();
    assert_eq!(
        req,
        Some(CursorRequest {
            x: 5,
            y: 0,
            shape: CursorShape::Bar
        })
    );
}

#[test]
fn input_line_no_cursor_when_unfocused() {
    let mut il = InputLine::new();
    il.set_bounds(Rect::new(0, 0, 40, 1));
    il.set_text("hello");

    assert_eq!(il.cursor(), None);
}

#[test]
fn cursor_propagates_through_split_pane() {
    let mut il = InputLine::new();
    il.set_text("ab");

    let mut sp = SplitPane::new(SplitDirection::Horizontal, Box::new(Dummy::new()), Box::new(il));
    sp.set_bounds(Rect::new(0, 0, 40, 10));
    // Focus the right child (InputLine is child 1)
    sp.focus_next();

    let req = sp.cursor();
    // InputLine cursor at x=2 within its own bounds, translated by its position in the split
    assert!(req.is_some(), "cursor should propagate through SplitPane");
    let r = req.unwrap();
    assert_eq!(r.shape, CursorShape::Bar);
    // x should be offset by the left pane width + separator
    assert!(r.x >= 2, "cursor x={} should include child offset", r.x);
}

#[test]
fn cursor_propagates_through_program_group() {
    let mut il = InputLine::new();
    il.set_text("xyz");

    let mut group = GroupState::default();
    group.set_bounds(Rect::new(0, 0, 80, 24));
    group.insert(Box::new(il));
    group.set_focused_index(0);
    group.select_focused();
    group.set_child_bounds(0, Rect::new(5, 10, 40, 1));

    let req = group.cursor();
    assert_eq!(
        req,
        Some(CursorRequest {
            x: 5 + 3,
            y: 10,
            shape: CursorShape::Bar
        })
    );
}

#[test]
fn cursor_propagates_through_input_dialog() {
    let mut dlg = InputDialog::new("Test");
    dlg.set_bounds(Rect::new(0, 0, 40, 5));
    dlg.select();

    let req = dlg.cursor();
    // InputLine is at offset (2, 2) inside the dialog, cursor at x=0
    assert!(req.is_some(), "cursor should propagate through InputDialog");
    let r = req.unwrap();
    assert_eq!(r.shape, CursorShape::Bar);
    assert_eq!(r.x, 2); // inner padding
    assert_eq!(r.y, 2); // row 2 inside dialog
}
