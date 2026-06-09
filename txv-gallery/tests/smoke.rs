//! Smoke tests: app builds, renders, shows widget list.

use txv_core::prelude::*;

#[test]
fn gallery_app_builds() {
    let app = txv_gallery::build_app();
    assert!(app.bounds().w() > 0);
    assert!(app.bounds().h() > 0);
}

#[test]
fn gallery_renders_widget_list() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(80, 24);
    app.set_bounds(Rect::new(0, 0, 80, 24));
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("StatusBar"), "should show first widget name");
    assert!(backend.contains("InputLine"), "should show second widget name");
    assert!(backend.contains("ListView"), "should show ListView in list");
}

#[test]
fn gallery_shows_status_bar() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(80, 24);
    app.set_bounds(Rect::new(0, 0, 80, 24));
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("q Quit"), "status bar should show quit hint");
}

#[test]
fn gallery_shows_code_snippet() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(80, 24);
    app.set_bounds(Rect::new(0, 0, 80, 24));
    run_cycles(&mut app, &mut backend, 1);
    // First demo is StatusBar — snippet should contain StatusBar setup code
    assert!(
        backend.contains("StatusBar::new"),
        "should show code snippet for StatusBar"
    );
}
