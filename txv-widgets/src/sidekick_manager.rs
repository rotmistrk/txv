//! SidekickManager — postprocess View that holds a shared popup View.
//!
//! When hidden: bounds 0×0, invisible.
//! On CM_SIDEKICK_SHOW: stores the Arc<Mutex<dyn View>>, positions itself.
//! On CM_SIDEKICK_HIDE: drops the Arc, goes invisible.
//! The owner (InputLine) mutates the shared View directly — no re-creation needed.

use std::sync::{Arc, Mutex};

use txv_core::prelude::*;

use crate::sidekick::{SidekickShow, CM_SIDEKICK_HIDE, CM_SIDEKICK_SHOW};

pub struct SidekickManager {
    state: ViewState,
    child: Option<Arc<Mutex<dyn View>>>,
}

impl SidekickManager {
    pub fn new() -> Self {
        Self {
            state: ViewState::new(ViewOptions {
                postprocess: true,
                ..ViewOptions::default()
            }),
            child: None,
        }
    }
}

impl View for SidekickManager {
    delegate_view_state!(state, override { draw, handle });

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        if let Some(arc) = &self.child {
            if let Ok(mut child) = arc.lock() {
                child.draw();
                self.state.buffer_mut().blit(child.buffer(), 0, 0);
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Command { id, data } = event else {
            return HandleResult::Ignored;
        };
        match *id {
            CM_SIDEKICK_SHOW => {
                if let Some(show) = data.as_ref().and_then(|d| d.downcast_ref::<SidekickShow>()) {
                    self.child = Some(Arc::clone(&show.view));
                    self.state.set_bounds(show.rect);
                    self.state.mark_dirty();
                }
                HandleResult::Consumed
            }
            CM_SIDEKICK_HIDE => {
                self.child = None;
                self.state.set_bounds(Rect::new(0, 0, 0, 0));
                self.state.mark_dirty();
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl Default for SidekickManager {
    fn default() -> Self {
        Self::new()
    }
}
