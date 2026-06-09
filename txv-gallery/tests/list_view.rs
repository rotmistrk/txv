//! ListView tests: navigate list, selection follows cursor.

use txv_core::prelude::*;

#[test]
fn list_view_demo_shows_items() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(80, 24);
    app.set_bounds(Rect::new(0, 0, 80, 24));

    // Navigate to ListView demo (index 4)
    for _ in 0..4 {
        backend.inject_key(KeyCode::Down, KeyMod::NONE);
    }
    run_cycles(&mut app, &mut backend, 1);

    // The ListView demo shows the same widget list items
    // and the code snippet mentions "ListView"
    assert!(
        backend.contains("ListView"),
        "should show ListView in widget list or snippet"
    );
}

#[test]
fn list_starts_at_first_item() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(80, 24);
    app.set_bounds(Rect::new(0, 0, 80, 24));
    run_cycles(&mut app, &mut backend, 1);

    // Widget list should show StatusBar as the first (highlighted) item
    assert!(backend.contains("StatusBar"), "first item should be StatusBar");
}

#[test]
fn list_end_key_goes_to_last() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 24);
    app.set_bounds(Rect::new(0, 0, 100, 24));
    run_cycles(&mut app, &mut backend, 1);

    // Press End to jump to last item
    backend.inject_key(KeyCode::End, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);

    // Last widget is FocusGatedGroup — snippet should reference it
    assert!(
        backend.contains("FocusGatedGroup"),
        "End should jump to last item (FocusGatedGroup)"
    );
}

#[test]
fn list_home_key_returns_to_first() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 24);
    app.set_bounds(Rect::new(0, 0, 100, 24));
    run_cycles(&mut app, &mut backend, 1);

    // Go to end, then Home
    backend.inject_key(KeyCode::End, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    backend.inject_key(KeyCode::Home, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);

    // Should show StatusBar snippet
    assert!(backend.contains("StatusBar::new"), "Home should return to first item");
}
