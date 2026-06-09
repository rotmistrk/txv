//! Integration test: InputLine completion with sidekick popup.

use txv_core::prelude::*;
use txv_widgets::sidekick::{SidekickRequest, CM_SIDEKICK_SHOW};
use txv_widgets::TextArea;

/// SidekickManager receives CM_SIDEKICK_SHOW via Gallery's group dispatch.
#[test]
fn sidekick_manager_receives_show_command() {
    let mut app = txv_gallery::build_app();
    app.set_bounds(Rect::new(0, 0, 100, 30));
    let sink = EventSink::new();
    app.set_sink(sink.clone());

    let mut popup = TextArea::new();
    popup.set_content("popup");
    let req = SidekickRequest::new(Box::new(popup), Rect::new(0, 0, 20, 5), 0);
    let ev = Event::Command {
        id: CM_SIDEKICK_SHOW,
        data: Some(Box::new(req)),
        broadcast: false,
    };

    app.handle(&ev);

    let gs = app.group_state().unwrap();
    let sk = gs.child(2).unwrap();
    let sk_gs = sk.group_state().unwrap();
    assert_eq!(sk_gs.child_count(), 1, "sidekick should have popup view");
}

/// Full flow: type in InputLine → completion → sidekick popup appears.
#[test]
fn input_line_completion_triggers_sidekick() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));

    // Navigate to InputLine, focus demo, type "S", press Tab
    backend.inject_key(KeyCode::Down, KeyMod::NONE);
    backend.inject_key(KeyCode::Right, KeyMod::CTRL.with_shift());
    backend.inject_key(KeyCode::Char('S'), KeyMod::NONE);
    backend.inject_key(KeyCode::Tab, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);

    // Sidekick should have a popup child
    let gs = app.group_state().unwrap();
    let sk = gs.child(2).unwrap();
    let sk_gs = sk.group_state().unwrap();
    assert_eq!(sk_gs.child_count(), 1, "sidekick should show completion popup");
}
