//! Scenario tests for completion dropdown (InputLine + SidekickManager + DropdownMenu).

use txv_core::prelude::*;

/// Navigate to InputLine demo (index 1), focus it.
fn nav_to_input(app: &mut dyn View, backend: &mut MockBackend) {
    backend.inject_key(KeyCode::Down, KeyMod::NONE);
    run_cycles(app, backend, 1);
    backend.inject_key(KeyCode::Right, KeyMod::CTRL.with_shift());
    run_cycles(app, backend, 1);
    // Clear default text
    backend.inject_key(KeyCode::Char('u'), KeyMod::CTRL);
    run_cycles(app, backend, 1);
}

#[test]
fn tab_opens_completion_dropdown() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_input(&mut app, &mut backend);
    // Type "S" then Tab — should show completion popup with StatusBar, SplitPane
    backend.inject_key(KeyCode::Char('S'), KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    backend.inject_key(KeyCode::Tab, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    // LCP is "S" so dropdown should appear with matches
    assert!(
        backend.contains("StatusBar") || backend.contains("SplitPane"),
        "completion items visible"
    );
}

#[test]
fn single_match_auto_completes() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_input(&mut app, &mut backend);
    // Type "Sta" then Tab — only "StatusBar" matches, auto-completes
    backend.inject_str("Sta");
    run_cycles(&mut app, &mut backend, 1);
    backend.inject_key(KeyCode::Tab, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("StatusBar"), "auto-completed to StatusBar");
}

#[test]
fn down_navigates_in_dropdown() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_input(&mut app, &mut backend);
    backend.inject_key(KeyCode::Char('S'), KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    backend.inject_key(KeyCode::Tab, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    // Press Down to move cursor in dropdown
    backend.inject_key(KeyCode::Down, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    // No crash, dropdown still visible
    assert!(
        backend.contains("StatusBar") || backend.contains("SplitPane"),
        "dropdown still visible after Down"
    );
}

#[test]
fn esc_closes_dropdown() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_input(&mut app, &mut backend);
    backend.inject_key(KeyCode::Char('S'), KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    backend.inject_key(KeyCode::Tab, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    // Esc should close completion
    backend.inject_key(KeyCode::Esc, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    // Dropdown gone (sidekick hidden)
    let gs = app.group_state().unwrap();
    let sk = gs.child(2).unwrap();
    let sk_gs = sk.group_state().unwrap();
    assert_eq!(sk_gs.child_count(), 0, "sidekick empty after Esc");
}

#[test]
fn no_matches_no_dropdown() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_input(&mut app, &mut backend);
    backend.inject_str("zzz");
    run_cycles(&mut app, &mut backend, 1);
    backend.inject_key(KeyCode::Tab, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    // No matches → no sidekick popup
    let gs = app.group_state().unwrap();
    let sk = gs.child(2).unwrap();
    let sk_gs = sk.group_state().unwrap();
    assert_eq!(sk_gs.child_count(), 0, "no dropdown for no matches");
}
