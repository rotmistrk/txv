use super::*;
use crate::event::{Event, KeyCode, KeyEvent, KeyMod};
use crate::view::HandleResult;

#[path = "test_dummy.rs"]
mod test_dummy;
#[path = "test_helpers.rs"]
mod test_helpers;
use test_dummy::DummyView;
use test_helpers::PreView;

#[test]
fn focus_next_skips_unfocusable() {
    let mut g = GroupState::default();
    g.insert(Box::new(DummyView::new(true)));
    g.insert(Box::new(DummyView::new(false)));
    g.insert(Box::new(DummyView::new(true)));
    g.children[0].select();
    g.focus_next();
    assert_eq!(g.focused, 2);
}

#[test]
fn focus_prev_wraps() {
    let mut g = GroupState::default();
    g.insert(Box::new(DummyView::new(true)));
    g.insert(Box::new(DummyView::new(true)));
    g.insert(Box::new(DummyView::new(true)));
    g.children[0].select();
    g.focus_prev();
    assert_eq!(g.focused, 2);
}

#[test]
fn three_phase_dispatch() {
    let mut g = GroupState::default();
    g.insert(Box::new(PreView::new()));
    g.insert(Box::new(DummyView::new(true)));
    g.focused = 1;

    let ev = Event::Key(KeyEvent {
        code: KeyCode::Char('x'),
        modifiers: KeyMod::default(),
    });
    let result = g.dispatch(&ev);
    assert_eq!(result, HandleResult::Consumed);
}
