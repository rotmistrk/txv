//! DummyView — test helper for group tests.

use crate::event::Event;
use crate::view::{HandleResult, View, ViewOptions, ViewState};

pub(super) struct DummyView {
    state: ViewState,
}

impl DummyView {
    pub(super) fn new(focusable: bool) -> Self {
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
