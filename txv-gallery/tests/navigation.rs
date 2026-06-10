//! Navigation tests: arrow keys move selection, center panel changes.

use txv_core::prelude::*;

#[test]
fn arrow_down_moves_list_selection() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(80, 24);
    app.set_bounds(Rect::new(0, 0, 80, 24));

    // Initial render
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("StatusBar"), "starts with StatusBar selected");

    // Press Down arrow to move to InputLine
    backend.inject_key(KeyCode::Down, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);

    // Demo should switch — snippet should now mention InputLine setup
    assert!(
        backend.contains("with_command"),
        "InputLine demo snippet should contain with_command"
    );
}

#[test]
fn arrow_up_from_second_returns_to_first() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 24);
    app.set_bounds(Rect::new(0, 0, 100, 24));
    run_cycles(&mut app, &mut backend, 1);

    // Down then Up
    backend.inject_key(KeyCode::Down, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);

    // Verify we switched to InputLine
    assert!(
        backend.contains("with_command"),
        "should show InputLine snippet after Down"
    );

    backend.inject_key(KeyCode::Up, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);

    // Should be back to StatusBar demo — check for 'bar' which is shorter
    assert!(backend.contains("StatusBar"), "should return to StatusBar snippet");
}

#[test]
fn ctrl_shift_right_switches_focus_between_panels() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(80, 24);
    app.set_bounds(Rect::new(0, 0, 80, 24));
    run_cycles(&mut app, &mut backend, 1);

    // Press Ctrl+Shift+Right to move focus from list to demo panel
    backend.inject_key(KeyCode::Right, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);

    // Press Down — should NOT change the widget list since focus is on demo
    backend.inject_key(KeyCode::Down, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);

    // Should still show StatusBar snippet (list didn't move)
    assert!(
        backend.contains("StatusBar::new"),
        "list should not move when demo panel is focused"
    );
}

#[test]
fn multiple_downs_navigate_through_list() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(80, 24);
    app.set_bounds(Rect::new(0, 0, 80, 24));
    run_cycles(&mut app, &mut backend, 1);

    // Navigate to ModalKey (index 2)
    backend.inject_key(KeyCode::Down, KeyMod::NONE);
    backend.inject_key(KeyCode::Down, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);

    // Snippet should show ModalKey setup
    assert!(
        backend.contains("ModalKey::new"),
        "should show ModalKey snippet after 2 downs"
    );
}
