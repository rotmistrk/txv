//! Integration tests for TiledWorkspace.

use txv_core::prelude::*;

use crate::tiled_workspace::commands::{CM_TW_FOCUS_RIGHT, CM_TW_LAYOUT_CYCLE, CM_TW_TOGGLE_TREE, CM_TW_ZOOM};
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
fn wide_layout_three_columns() -> Result<(), Box<dyn std::error::Error>> {
    let mut ws = three_panel_workspace();
    ws.insert_tab(0, "Files", Box::new(Dummy::new()));
    ws.insert_tab(1, "Editor", Box::new(Dummy::new()));
    ws.insert_tab(2, "Shell", Box::new(Dummy::new()));
    ws.set_bounds(Rect::new(0, 0, 200, 50));

    let c0 = ws.group.child(0).ok_or("child 0")?;
    let b0 = c0.bounds();
    let c1 = ws.group.child(1).ok_or("child 1")?;
    let b1 = c1.bounds();
    let c2 = ws.group.child(2).ok_or("child 2")?;
    let b2 = c2.bounds();
    let (o0x, _) = ws.group.child_origin(0);
    let (o1x, _) = ws.group.child_origin(1);
    let (o2x, _) = ws.group.child_origin(2);

    assert!(b0.w() > 0, "tree should have width");
    assert!(b1.w() > 0, "main should have width");
    assert!(b2.w() > 0, "tools should have width");
    // 2 divider gaps (1 cell each) between 3 panels
    assert_eq!(b0.w() + b1.w() + b2.w() + 2, 200);
    assert!(o0x < o1x, "tree left of main");
    assert!(o1x < o2x, "main left of tools");
    Ok(())
}

#[test]
fn narrow_layout_stacked() -> Result<(), Box<dyn std::error::Error>> {
    let mut ws = three_panel_workspace();
    ws.layout_mode = LayoutMode::Narrow;
    ws.insert_tab(0, "Files", Box::new(Dummy::new()));
    ws.insert_tab(1, "Editor", Box::new(Dummy::new()));
    ws.insert_tab(2, "Shell", Box::new(Dummy::new()));
    ws.set_bounds(Rect::new(0, 0, 80, 40));

    let c0 = ws.group.child(0).ok_or("child 0")?;
    let b0 = c0.bounds();
    let c1 = ws.group.child(1).ok_or("child 1")?;
    let b1 = c1.bounds();
    let c2 = ws.group.child(2).ok_or("child 2")?;
    let b2 = c2.bounds();
    let (_, o1y) = ws.group.child_origin(1);
    let (_, o2y) = ws.group.child_origin(2);

    // Narrow: tree on left, main+tools stacked vertically on right
    assert!(b0.w() > 0);
    assert!(b1.w() > 0);
    assert!(b2.w() > 0);
    assert!(o1y < o2y, "main above tools in narrow mode");
    Ok(())
}

#[test]
fn toggle_hides_panel() -> Result<(), Box<dyn std::error::Error>> {
    let mut ws = three_panel_workspace();
    ws.insert_tab(0, "Files", Box::new(Dummy::new()));
    ws.insert_tab(1, "Editor", Box::new(Dummy::new()));
    ws.insert_tab(2, "Shell", Box::new(Dummy::new()));
    ws.set_bounds(Rect::new(0, 0, 200, 50));

    ws.toggle_panel(0); // hide tree
    let c0 = ws.group.child(0).ok_or("child 0")?;
    let b0 = c0.bounds();
    let (o1x, _) = ws.group.child_origin(1);
    // Tree gets no space, main+tools fill width
    assert_eq!(b0.w(), 0);
    assert_eq!(o1x, 0, "main starts at left edge when tree hidden");
    Ok(())
}

#[test]
fn zoom_gives_full_bounds() -> Result<(), Box<dyn std::error::Error>> {
    let mut ws = three_panel_workspace();
    ws.insert_tab(1, "Editor", Box::new(Dummy::new()));
    ws.set_bounds(Rect::new(0, 0, 200, 50));
    ws.focus_panel(1);
    ws.toggle_zoom();

    let c1 = ws.group.child(1).ok_or("child 1")?;
    let b1 = c1.bounds();
    assert_eq!(b1.w(), 200);
    assert_eq!(b1.h(), 50);
    Ok(())
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

    ws.focus_direction(1, 0); // right wraps to tree
    assert_eq!(ws.group.focused_index(), 0, "should wrap to tree");

    ws.focus_direction(-1, 0); // left wraps to tools
    assert_eq!(ws.group.focused_index(), 2, "should wrap to tools");
}

#[test]
fn layout_cycle_changes_mode() {
    let mut ws = three_panel_workspace();
    ws.is_wide = false; // start narrow for 80-col terminal
    ws.insert_tab(0, "Files", Box::new(Dummy::new()));
    ws.insert_tab(1, "Editor", Box::new(Dummy::new()));
    ws.insert_tab(2, "Shell", Box::new(Dummy::new()));
    ws.set_bounds(Rect::new(0, 0, 80, 40)); // narrow by threshold

    // Auto mode: narrow terminal → narrow layout (tools below)
    let (_, o2y) = ws.group.child_origin(2);
    let (_, o1y) = ws.group.child_origin(1);
    assert!(o2y > o1y, "narrow auto: tools below main");

    // Force wide
    ws.cycle_layout(); // Auto → Wide
    let (_, o2y) = ws.group.child_origin(2);
    let (_, o1y) = ws.group.child_origin(1);
    assert_eq!(o2y, o1y, "forced wide: tools beside main");

    // Force narrow
    ws.cycle_layout(); // Wide → Narrow
    ws.set_bounds(Rect::new(0, 0, 200, 50)); // wide terminal but forced narrow
    let (_, o2y) = ws.group.child_origin(2);
    let (_, o1y) = ws.group.child_origin(1);
    assert!(o2y > o1y, "forced narrow: tools below even on wide terminal");

    // Back to auto
    ws.cycle_layout(); // Narrow → Auto
    let (_, o2y) = ws.group.child_origin(2);
    let (_, o1y) = ws.group.child_origin(1);
    assert_eq!(o2y, o1y, "auto on wide terminal: tools beside");
}

#[test]
fn command_events_control_workspace() {
    let mut ws = three_panel_workspace();
    ws.insert_tab(0, "Files", Box::new(Dummy::new()));
    ws.insert_tab(1, "Editor", Box::new(Dummy::new()));
    ws.set_bounds(Rect::new(0, 0, 200, 50));

    // Focus via command (focus right)
    ws.handle_command(CM_TW_FOCUS_RIGHT, &None);
    assert_ne!(ws.group.focused_index(), 0);

    // Toggle tree panel via command
    ws.handle_command(CM_TW_TOGGLE_TREE, &None);
    assert!(ws.hidden[0]);

    // Zoom via command
    ws.handle_command(CM_TW_ZOOM, &None);
    assert!(ws.zoomed.is_some());

    // Layout cycle via command
    ws.handle_command(CM_TW_LAYOUT_CYCLE, &None);
    assert_eq!(ws.layout_mode, LayoutMode::Wide);
}

#[test]
fn handle_keys_disabled_ignores_keystrokes() {
    let mut ws = three_panel_workspace();
    ws.insert_tab(0, "Files", Box::new(Dummy::new()));
    ws.insert_tab(1, "Editor", Box::new(Dummy::new()));
    ws.set_bounds(Rect::new(0, 0, 200, 50));
    ws.set_handle_keys(false);

    // M-/ (zoom) should NOT be consumed when keys disabled
    let zoom_key = Event::Key(KeyEvent::new(KeyCode::Char('/'), KeyMod::ALT));
    let result = ws.handle(&zoom_key);
    assert_eq!(result, HandleResult::Ignored);
    assert!(ws.zoomed.is_none(), "zoom should not trigger with keys disabled");
}

#[test]
fn default_bindings_returns_entries() {
    let ws = three_panel_workspace();
    let bindings = ws.default_bindings();
    assert!(bindings.len() >= 15, "should have at least 15 bindings");
}
