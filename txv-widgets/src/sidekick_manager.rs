//! SidekickManager — postprocess View that holds a shared popup View.
//!
//! When hidden: bounds 0×0, invisible.
//! On CM_SIDEKICK_SHOW: stores the Arc<Mutex<dyn View>>, emits CM_REPOSITION.
//! On CM_SIDEKICK_HIDE: drops the Arc, repositions to 0×0.
//! The owner (InputLine) mutates the shared View directly — no re-creation needed.

use std::sync::{Arc, Mutex};

use txv_core::commands::{RepositionRequest, CM_REPOSITION};
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

    fn request_reposition(&self, width: u16, height: u16, offset_x: i16, offset_y: i16, relative_to: Option<ViewId>) {
        self.state.put_command(
            CM_REPOSITION,
            Some(Box::new(RepositionRequest {
                view_id: self.state.id(),
                width,
                height,
                offset_x,
                offset_y,
                relative_to,
            })),
        );
    }
}

impl View for SidekickManager {
    delegate_view_state!(state, override { draw, handle, needs_redraw, view_id });

    fn view_id(&self) -> ViewId {
        self.state.id()
    }

    fn needs_redraw(&self) -> bool {
        if self.state.is_dirty() {
            return true;
        }
        if let Some(arc) = &self.child {
            if let Ok(child) = arc.lock() {
                return child.needs_redraw();
            }
        }
        false
    }

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        if let Some(arc) = &self.child {
            if let Ok(mut child) = arc.lock() {
                child.draw();
                child.mark_redrawn();
                self.state.buffer_mut().blit(child.buffer(), 0, 0);
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Command { id, data, .. } = event else {
            return HandleResult::Ignored;
        };
        match *id {
            CM_SIDEKICK_SHOW => {
                if let Some(show) = data.as_ref().and_then(|d| d.downcast_ref::<SidekickShow>()) {
                    let h = show.rect.h;
                    let w = show.rect.w;
                    if let Ok(mut child) = show.view.lock() {
                        child.set_bounds(Rect::new(0, 0, w, h));
                    }
                    self.child = Some(Arc::clone(&show.view));
                    // Position above the emitter
                    self.request_reposition(w, h, show.rect.x as i16, -(h as i16), Some(show.emitter_id));
                }
                HandleResult::Consumed
            }
            CM_SIDEKICK_HIDE => {
                self.child = None;
                self.request_reposition(0, 0, 0, 0, None);
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
