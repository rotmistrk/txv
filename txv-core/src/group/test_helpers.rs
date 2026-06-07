//! PreView — test helper for group tests (preprocess view).

use crate::event::Event;
use crate::view::{HandleResult, View, ViewOptions, ViewState};

pub(super) struct PreView {
    state: ViewState,
}

impl PreView {
    pub(super) fn new() -> Self {
        Self {
            state: ViewState::new(ViewOptions {
                preprocess: true,
                focusable: false,
                ..ViewOptions::default()
            }),
        }
    }
}

impl View for PreView {
    crate::delegate_view_state!(state);
    fn draw(&mut self) {}
    fn handle(&mut self, _event: &Event) -> HandleResult {
        HandleResult::Consumed
    }
}
