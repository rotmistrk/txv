//! Resize tests: Alt+Shift+Arrows resize panes.

use txv_core::prelude::*;

#[test]
fn alt_shift_right_grows_left_panel() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    run_cycles(&mut app, &mut backend, 1);

    // Measure initial left panel width (find first │ separator)
    let row1_before = backend.row(1);
    let sep_before = row1_before.find('│').unwrap_or(0);

    // Alt+Shift+Right should grow the focused (left) panel
    backend.inject_key(KeyCode::Right, KeyMod::ALT.with_shift());
    run_cycles(&mut app, &mut backend, 1);

    let row1_after = backend.row(1);
    let sep_after = row1_after.find('│').unwrap_or(0);

    assert!(
        sep_after > sep_before,
        "left panel should grow: before={sep_before} after={sep_after}"
    );
}

#[test]
fn alt_shift_left_shrinks_left_panel() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    run_cycles(&mut app, &mut backend, 1);

    let row1_before = backend.row(1);
    let sep_before = row1_before.find('│').unwrap_or(0);

    backend.inject_key(KeyCode::Left, KeyMod::ALT.with_shift());
    run_cycles(&mut app, &mut backend, 1);

    let row1_after = backend.row(1);
    let sep_after = row1_after.find('│').unwrap_or(0);

    assert!(
        sep_after < sep_before,
        "left panel should shrink: before={sep_before} after={sep_after}"
    );
}

#[test]
fn terminal_resize_relayouts() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    run_cycles(&mut app, &mut backend, 1);

    // Resize terminal to 120x40
    backend.set_size(120, 40);
    run_cycles(&mut app, &mut backend, 1);

    // Should still render correctly with wider layout
    assert!(backend.contains("StatusBar"), "widget list visible after resize");
    assert!(backend.contains("q Quit"), "status bar visible after resize");

    // Status bar should be on last row (39)
    let last_row = backend.row(39);
    assert!(last_row.contains("q Quit"), "status bar on new last row");
}
