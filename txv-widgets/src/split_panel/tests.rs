//! Tests for SplitPanel.

use txv_core::prelude::*;

use crate::split_panel::SplitPanel;
use crate::tiled_workspace::types::SplitDir;

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
fn horizontal_split_divides_width() {
    let mut sp = SplitPanel::new(SplitDir::Horizontal);
    sp.add_child(Box::new(Dummy::new()), 0.5);
    sp.add_child(Box::new(Dummy::new()), 0.5);
    sp.set_bounds(Rect::new(0, 0, 100, 40));

    let b0 = sp.child(0).unwrap().bounds();
    let b1 = sp.child(1).unwrap().bounds();
    // 1 column reserved for divider
    assert_eq!(b0.w + b1.w + 1, 100);
    assert_eq!(b0.h, 40);
    assert_eq!(b1.h, 40);
    assert_eq!(b0.x, 0);
    assert!(b1.x > 0);
}

#[test]
fn vertical_split_divides_height() {
    let mut sp = SplitPanel::new(SplitDir::Vertical);
    sp.add_child(Box::new(Dummy::new()), 0.6);
    sp.add_child(Box::new(Dummy::new()), 0.4);
    sp.set_bounds(Rect::new(0, 0, 80, 40));

    let b0 = sp.child(0).unwrap().bounds();
    let b1 = sp.child(1).unwrap().bounds();
    // Second child starts where first ends (overlaps divider row)
    assert_eq!(b1.y, b0.y + b0.h, "second child starts at divider row");
    assert_eq!(b0.h + b1.h, 40, "children fill full height");
    assert_eq!(b0.w, 80);
    assert_eq!(b0.y, 0);
}

#[test]
fn set_direction_relayouts() {
    let mut sp = SplitPanel::new(SplitDir::Horizontal);
    sp.add_child(Box::new(Dummy::new()), 0.5);
    sp.add_child(Box::new(Dummy::new()), 0.5);
    sp.set_bounds(Rect::new(0, 0, 80, 40));

    // Horizontal: children side by side
    let b0 = sp.child(0).unwrap().bounds();
    assert_eq!(b0.h, 40);

    // Switch to vertical
    sp.set_direction(SplitDir::Vertical);
    let b0 = sp.child(0).unwrap().bounds();
    assert_eq!(b0.w, 80);
    assert!(b0.h > 0 && b0.h < 40, "should split height");
}

#[test]
fn cycle_focus_wraps() {
    let mut sp = SplitPanel::new(SplitDir::Horizontal);
    sp.add_child(Box::new(Dummy::new()), 0.5);
    sp.add_child(Box::new(Dummy::new()), 0.5);

    assert_eq!(sp.focused_index(), 0);
    sp.cycle_focus();
    assert_eq!(sp.focused_index(), 1);
    sp.cycle_focus();
    assert_eq!(sp.focused_index(), 0);
}

#[test]
fn grow_shrink_adjusts_proportions() {
    let mut sp = SplitPanel::new(SplitDir::Horizontal);
    sp.add_child(Box::new(Dummy::new()), 0.5);
    sp.add_child(Box::new(Dummy::new()), 0.5);
    sp.set_bounds(Rect::new(0, 0, 100, 40));

    let before = sp.child(0).unwrap().bounds().w;
    sp.grow_focused();
    let after = sp.child(0).unwrap().bounds().w;
    assert!(after > before);

    sp.shrink_focused();
    let after2 = sp.child(0).unwrap().bounds().w;
    assert!(after2 < after);
}

#[test]
fn remove_child_adjusts_focus() {
    let mut sp = SplitPanel::new(SplitDir::Horizontal);
    sp.add_child(Box::new(Dummy::new()), 0.33);
    sp.add_child(Box::new(Dummy::new()), 0.33);
    sp.add_child(Box::new(Dummy::new()), 0.34);
    sp.set_focused(2);

    sp.remove_child(0);
    assert_eq!(sp.child_count(), 2);
    assert_eq!(sp.focused_index(), 1); // was 2, shifted
}

#[test]
fn needs_redraw_propagates_from_children() {
    let mut sp = SplitPanel::new(SplitDir::Horizontal);
    sp.add_child(Box::new(Dummy::new()), 0.5);
    sp.add_child(Box::new(Dummy::new()), 0.5);
    sp.set_bounds(Rect::new(0, 0, 80, 40));

    // After set_bounds, panel is dirty
    assert!(sp.needs_redraw());
    sp.mark_redrawn();
    assert!(!sp.needs_redraw());

    // Mark a child dirty — parent should report needs_redraw
    sp.child_mut(0).unwrap().set_bounds(Rect::new(0, 0, 39, 40));
    assert!(sp.needs_redraw());
}
