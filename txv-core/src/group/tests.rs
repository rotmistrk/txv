use super::*;
use crate::event::{Event, KeyCode, KeyEvent, KeyMod};
use crate::view::{HandleResult, ViewState};

struct DummyView {
    state: ViewState,
}
impl DummyView {
    fn new(focusable: bool) -> Self {
        Self {
            state: ViewState::new(ViewOptions {
                focusable,
                ..ViewOptions::default()
            }),
        }
    }
}
impl View for DummyView {
    crate::delegate_view_state!(state);
    fn draw(&mut self) {}
    fn handle(&mut self, _event: &Event) -> HandleResult {
        HandleResult::Ignored
    }
}

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
    struct PreView {
        state: ViewState,
    }
    impl View for PreView {
        crate::delegate_view_state!(state);
        fn draw(&mut self) {}
        fn handle(&mut self, _event: &Event) -> HandleResult {
            HandleResult::Consumed
        }
    }

    let mut g = GroupState::default();
    g.insert(Box::new(PreView {
        state: ViewState::new(ViewOptions {
            preprocess: true,
            focusable: false,
            ..ViewOptions::default()
        }),
    }));
    g.insert(Box::new(DummyView::new(true)));
    g.focused = 1;

    let ev = Event::Key(KeyEvent {
        code: KeyCode::Char('x'),
        modifiers: KeyMod::default(),
    });
    let result = g.dispatch(&ev);
    assert_eq!(result, HandleResult::Consumed);
}
