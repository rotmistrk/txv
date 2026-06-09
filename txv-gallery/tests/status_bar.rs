//! StatusBar tests: items visible, correct positions.

use txv_core::prelude::*;

#[test]
fn status_bar_shows_quit_hint() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(80, 24);
    app.set_bounds(Rect::new(0, 0, 80, 24));
    run_cycles(&mut app, &mut backend, 1);

    // Status bar is on the last row
    let last_row = backend.row(23);
    assert!(
        last_row.contains("q Quit"),
        "last row should contain quit hint, got: {last_row}"
    );
}

#[test]
fn status_bar_shows_navigate_hint() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(80, 24);
    app.set_bounds(Rect::new(0, 0, 80, 24));
    run_cycles(&mut app, &mut backend, 1);

    let last_row = backend.row(23);
    assert!(
        last_row.contains("Navigate"),
        "status bar should show navigate hint, got: {last_row}"
    );
}

#[test]
fn status_bar_on_bottom_row_only() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(80, 24);
    app.set_bounds(Rect::new(0, 0, 80, 24));
    run_cycles(&mut app, &mut backend, 1);

    // "q Quit" should not appear in content area (rows 0..23)
    assert!(
        !backend.content_contains("q Quit"),
        "quit hint should only be in status bar, not content"
    );
}
