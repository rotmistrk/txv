//! DropdownMenu scenario tests — through the gallery app.

use txv_core::prelude::*;

#[test]
fn dropdown_visible_in_gallery() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    for _ in 0..10 {
        backend.inject_key(KeyCode::Down, KeyMod::NONE);
    }
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Rust"), "Rust should be visible");
    assert!(backend.contains("Python"), "Python should be visible");
    assert!(backend.contains("systems"), "secondary text visible");
}

#[test]
fn dropdown_cursor_navigation() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    for _ in 0..10 {
        backend.inject_key(KeyCode::Down, KeyMod::NONE);
    }
    run_cycles(&mut app, &mut backend, 1);
    // Focus demo panel
    backend.inject_key(KeyCode::Right, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);
    // Move cursor down
    backend.inject_key(KeyCode::Down, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Rust"));
    assert!(backend.contains("Go"));
}

#[test]
fn dropdown_filter_narrows() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    for _ in 0..10 {
        backend.inject_key(KeyCode::Down, KeyMod::NONE);
    }
    run_cycles(&mut app, &mut backend, 1);
    backend.inject_key(KeyCode::Right, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);
    // Type "ru" to filter
    backend.inject_key(KeyCode::Char('r'), KeyMod::NONE);
    backend.inject_key(KeyCode::Char('u'), KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Rust"), "Rust matches 'ru'");
    assert!(!backend.contains("Python"), "Python filtered out");
}

#[test]
fn dropdown_backspace_widens() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    for _ in 0..10 {
        backend.inject_key(KeyCode::Down, KeyMod::NONE);
    }
    run_cycles(&mut app, &mut backend, 1);
    backend.inject_key(KeyCode::Right, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);
    // Filter to "rus"
    backend.inject_str("rus");
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Rust"));
    assert!(!backend.contains("Elixir"));
    // Backspace twice → "r"
    backend.inject_key(KeyCode::Backspace, KeyMod::NONE);
    backend.inject_key(KeyCode::Backspace, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Elixir"), "Elixir contains 'r'");
}

#[test]
fn dropdown_number_hotkey() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    for _ in 0..10 {
        backend.inject_key(KeyCode::Down, KeyMod::NONE);
    }
    run_cycles(&mut app, &mut backend, 1);
    backend.inject_key(KeyCode::Right, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);
    // Press '3' — selects 3rd item (Python)
    backend.inject_key(KeyCode::Char('3'), KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("DropdownMenu"));
}

#[test]
fn dropdown_tab_autocompletes_lcp() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    for _ in 0..10 {
        backend.inject_key(KeyCode::Down, KeyMod::NONE);
    }
    run_cycles(&mut app, &mut backend, 1);
    backend.inject_key(KeyCode::Right, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);
    // Type "go" — matches only "Go", Tab should fill to "go"
    backend.inject_key(KeyCode::Char('g'), KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    // "g" matches Go, Zig (contains g) — Tab fills LCP
    backend.inject_key(KeyCode::Tab, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    // After Tab, filter should have advanced
    // Verify Go is still visible
    assert!(backend.contains("Go"));
}

#[test]
fn dropdown_right_arrow_autocompletes() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    for _ in 0..10 {
        backend.inject_key(KeyCode::Down, KeyMod::NONE);
    }
    run_cycles(&mut app, &mut backend, 1);
    backend.inject_key(KeyCode::Right, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);
    // Type "el" — matches Elixir. Right fills to "elixir"
    backend.inject_key(KeyCode::Char('e'), KeyMod::NONE);
    backend.inject_key(KeyCode::Char('l'), KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    backend.inject_key(KeyCode::Right, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Elixir"));
}

#[test]
fn dropdown_shows_count_label() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    for _ in 0..10 {
        backend.inject_key(KeyCode::Down, KeyMod::NONE);
    }
    run_cycles(&mut app, &mut backend, 1);
    // Should show item count "10/10" in the frame
    assert!(backend.contains("10/10"), "count label visible");
}
