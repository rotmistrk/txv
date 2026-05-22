//! Integration tests for TiledWorkspace.

use txv_core::prelude::*;

use crate::tiled_workspace::types::*;
use crate::tiled_workspace::TiledWorkspace;

fn three_panel_workspace() -> TiledWorkspace {
    let configs = vec![
        PanelConfig::fixed("Tree", PanelPosition::Left),
        PanelConfig::new("Main", PanelPosition::Center),
        PanelConfig::new("Tools", PanelPosition::Right),
    ];
    let wide = SplitNode::h(vec![
        (0.2, SplitNode::leaf(0)),
        (0.5, SplitNode::leaf(1)),
        (0.3, SplitNode::leaf(2)),
    ]);
    let narrow = SplitNode::h(vec![
        (0.25, SplitNode::leaf(0)),
        (
            0.75,
            SplitNode::v(vec![(0.6, SplitNode::leaf(1)), (0.4, SplitNode::leaf(2))]),
        ),
    ]);
    TiledWorkspace::new(configs, wide, narrow, 120)
}

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
fn wide_layout_three_columns() {
    let mut ws = three_panel_workspace();
    ws.insert_tab(0, "Files", Box::new(Dummy::new()));
    ws.insert_tab(1, "Editor", Box::new(Dummy::new()));
    ws.insert_tab(2, "Shell", Box::new(Dummy::new()));
    ws.set_bounds(Rect::new(0, 0, 200, 50));

    let b0 = ws.group.child(0).unwrap().bounds();
    let b1 = ws.group.child(1).unwrap().bounds();
    let b2 = ws.group.child(2).unwrap().bounds();

    assert!(b0.w > 0, "tree should have width");
    assert!(b1.w > 0, "main should have width");
    assert!(b2.w > 0, "tools should have width");
    assert_eq!(b0.w + b1.w + b2.w, 200);
    assert!(b0.x < b1.x, "tree left of main");
    assert!(b1.x < b2.x, "main left of tools");
}

#[test]
fn narrow_layout_stacked() {
    let mut ws = three_panel_workspace();
    ws.insert_tab(0, "Files", Box::new(Dummy::new()));
    ws.insert_tab(1, "Editor", Box::new(Dummy::new()));
    ws.insert_tab(2, "Shell", Box::new(Dummy::new()));
    ws.set_bounds(Rect::new(0, 0, 80, 40));

    let b0 = ws.group.child(0).unwrap().bounds();
    let b1 = ws.group.child(1).unwrap().bounds();
    let b2 = ws.group.child(2).unwrap().bounds();

    // Narrow: tree on left, main+tools stacked vertically on right
    assert!(b0.w > 0);
    assert!(b1.w > 0);
    assert!(b2.w > 0);
    assert!(b1.y < b2.y, "main above tools in narrow mode");
}

#[test]
fn toggle_hides_panel() {
    let mut ws = three_panel_workspace();
    ws.insert_tab(0, "Files", Box::new(Dummy::new()));
    ws.insert_tab(1, "Editor", Box::new(Dummy::new()));
    ws.insert_tab(2, "Shell", Box::new(Dummy::new()));
    ws.set_bounds(Rect::new(0, 0, 200, 50));

    ws.toggle_panel(0); // hide tree
    let b0 = ws.group.child(0).unwrap().bounds();
    let b1 = ws.group.child(1).unwrap().bounds();
    // Tree gets no space, main+tools fill width
    assert_eq!(b0.w, 0);
    assert_eq!(b1.x, 0, "main starts at left edge when tree hidden");
}

#[test]
fn zoom_gives_full_bounds() {
    let mut ws = three_panel_workspace();
    ws.insert_tab(1, "Editor", Box::new(Dummy::new()));
    ws.set_bounds(Rect::new(0, 0, 200, 50));
    ws.focus_panel(1);
    ws.toggle_zoom();

    let b1 = ws.group.child(1).unwrap().bounds();
    assert_eq!(b1, Rect::new(0, 0, 200, 50));
}

#[test]
fn save_restore_preserves_state() {
    let mut ws = three_panel_workspace();
    ws.insert_tab(0, "Files", Box::new(Dummy::new()));
    ws.insert_tab(1, "Editor", Box::new(Dummy::new()));
    ws.set_bounds(Rect::new(0, 0, 200, 50));
    ws.toggle_panel(0); // hide tree

    let state = ws.save_state();
    assert!(state.hidden.contains(&0));

    // Restore into fresh workspace
    let mut ws2 = three_panel_workspace();
    ws2.insert_tab(0, "Files", Box::new(Dummy::new()));
    ws2.insert_tab(1, "Editor", Box::new(Dummy::new()));
    ws2.set_bounds(Rect::new(0, 0, 200, 50));
    ws2.restore_state(&state);
    assert!(ws2.hidden[0]);
}

#[test]
fn focus_direction_spatial() {
    let mut ws = three_panel_workspace();
    ws.insert_tab(0, "Files", Box::new(Dummy::new()));
    ws.insert_tab(1, "Editor", Box::new(Dummy::new()));
    ws.insert_tab(2, "Shell", Box::new(Dummy::new()));
    ws.set_bounds(Rect::new(0, 0, 200, 50));
    ws.focus_panel(0); // start at tree (leftmost)

    ws.focus_direction(1, 0); // right
    assert_eq!(ws.group.focused_index(), 1, "should focus main");

    ws.focus_direction(1, 0); // right again
    assert_eq!(ws.group.focused_index(), 2, "should focus tools");
}
