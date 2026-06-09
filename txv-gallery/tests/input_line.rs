//! InputLine tests: activate ModalKey, type text, confirm/cancel.

use txv_core::prelude::*;

#[test]
fn input_line_demo_shows_default_text() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(80, 24);
    app.set_bounds(Rect::new(0, 0, 80, 24));

    // Navigate to InputLine demo (index 1)
    backend.inject_key(KeyCode::Down, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);

    assert!(backend.contains("Type here"), "InputLine demo should show default text");
}

#[test]
fn modal_key_demo_shows_idle_label() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(80, 24);
    app.set_bounds(Rect::new(0, 0, 80, 24));

    // Navigate to ModalKey demo (index 2)
    backend.inject_key(KeyCode::Down, KeyMod::NONE);
    backend.inject_key(KeyCode::Down, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);

    assert!(backend.contains("F2 Go"), "ModalKey demo should show idle label");
}

#[test]
fn input_line_demo_accepts_typing_when_focused() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));

    // Navigate to InputLine demo
    backend.inject_key(KeyCode::Down, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);

    // Ctrl+Shift+Right to focus the demo panel
    backend.inject_key(KeyCode::Right, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);

    // Type into the InputLine (use text that won't trigger completion)
    backend.inject_str("xyz");
    run_cycles(&mut app, &mut backend, 1);

    assert!(backend.contains("xyz"), "typed text should appear in InputLine");
}
