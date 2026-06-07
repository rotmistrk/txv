//! Test helper: Dummy view for palette integration tests.

use txv_core::prelude::*;

pub(crate) struct Dummy {
    state: ViewState,
}

impl Dummy {
    pub(crate) fn new() -> Self {
        Self {
            state: ViewState::default(),
        }
    }
}

impl View for Dummy {
    delegate_view_state!(state);
    fn draw(&mut self) {}
    fn handle(&mut self, _: &Event) -> HandleResult {
        HandleResult::Ignored
    }
}
