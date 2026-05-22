//! Tests for ToolsPanel subpanel operations.

use txv_core::prelude::*;

use crate::tiled_workspace::types::SplitDir;
use crate::tools_panel::ToolsPanel;

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
fn starts_with_one_subpanel() {
    let tp = ToolsPanel::new(SplitDir::Horizontal);
    assert_eq!(tp.subpanel_count(), 1);
}

#[test]
fn split_on_move_creates_second_subpanel() {
    let mut tp = ToolsPanel::new(SplitDir::Horizontal);
    tp.set_bounds(Rect::new(0, 0, 80, 40));
    tp.insert_tab("Shell", Box::new(Dummy::new()));
    tp.insert_tab("Build", Box::new(Dummy::new()));

    assert_eq!(tp.subpanel_count(), 1);
    assert_eq!(tp.tab_count(), 2);

    tp.move_tab_to_next(); // split-on-move
    assert_eq!(tp.subpanel_count(), 2);
    assert_eq!(tp.subpanels[0].tab_count(), 1);
    assert_eq!(tp.subpanels[1].tab_count(), 1);
}

#[test]
fn auto_unsplit_when_empty() {
    let mut tp = ToolsPanel::new(SplitDir::Horizontal);
    tp.set_bounds(Rect::new(0, 0, 80, 40));
    tp.insert_tab("Shell", Box::new(Dummy::new()));
    tp.insert_tab("Build", Box::new(Dummy::new()));

    tp.move_tab_to_next(); // now 2 subpanels, 1 tab each
    assert_eq!(tp.subpanel_count(), 2);

    // Move the tab back — source becomes empty → auto-unsplit
    tp.focused = 1;
    tp.move_tab_to_next(); // wraps to subpanel 0
    assert_eq!(tp.subpanel_count(), 1);
    assert_eq!(tp.tab_count(), 2);
}

#[test]
fn cycle_focus_wraps() {
    let mut tp = ToolsPanel::new(SplitDir::Vertical);
    tp.set_bounds(Rect::new(0, 0, 80, 40));
    tp.insert_tab("A", Box::new(Dummy::new()));
    tp.insert_tab("B", Box::new(Dummy::new()));
    tp.move_tab_to_next(); // create split

    assert_eq!(tp.focused, 0);
    tp.cycle_focus();
    assert_eq!(tp.focused, 1);
    tp.cycle_focus();
    assert_eq!(tp.focused, 0); // wraps
}

#[test]
fn grow_shrink_adjusts_proportions() {
    let mut tp = ToolsPanel::new(SplitDir::Horizontal);
    tp.set_bounds(Rect::new(0, 0, 80, 40));
    tp.insert_tab("A", Box::new(Dummy::new()));
    tp.insert_tab("B", Box::new(Dummy::new()));
    tp.move_tab_to_next(); // 50/50 split

    let before = tp.proportions[0];
    tp.grow_focused();
    assert!(tp.proportions[0] > before, "grow should increase proportion");

    let before = tp.proportions[0];
    tp.shrink_focused();
    assert!(tp.proportions[0] < before, "shrink should decrease proportion");
}

#[test]
fn no_move_from_empty_subpanel() {
    let mut tp = ToolsPanel::new(SplitDir::Horizontal);
    tp.set_bounds(Rect::new(0, 0, 80, 40));
    // No tabs — move should be a no-op
    tp.move_tab_to_next();
    assert_eq!(tp.subpanel_count(), 1);
}
