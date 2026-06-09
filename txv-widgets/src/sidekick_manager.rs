//! SidekickManager — postprocess group that hosts a popup View.
//!
//! On CM_SIDEKICK_SHOW: takes ownership of view as child 0, emits CM_REPOSITION.
//! On CM_SIDEKICK_HIDE: removes child 0, repositions to 0×0.
//! Forwards CM_SIDEKICK_NEXT/PREV/APPLY to the child.

use txv_core::commands::{RepositionRequest, CM_REPOSITION};
use txv_core::prelude::*;

use crate::sidekick::{
    SidekickRequest, CM_SIDEKICK_APPLY, CM_SIDEKICK_HIDE, CM_SIDEKICK_NEXT, CM_SIDEKICK_PREV, CM_SIDEKICK_SHOW,
};

pub struct SidekickManager {
    group: GroupState,
}

impl SidekickManager {
    pub fn new() -> Self {
        Self {
            group: GroupState::new(ViewOptions::default().with_postprocess()),
        }
    }

    fn request_reposition(&self, width: u16, height: u16, offset_x: i16, offset_y: i16, relative_to: Option<ViewId>) {
        let my_id = self.view_id();
        let mut req = RepositionRequest::new(my_id, width, height).with_offset(offset_x, offset_y);
        if let Some(rel) = relative_to {
            req = req.relative_to(rel);
        }
        self.group.put_command(CM_REPOSITION, Some(Box::new(req)));
    }
}

impl View for SidekickManager {
    delegate_group_state!(group, override { draw, handle });

    fn draw(&mut self) {}

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Command { id, data, .. } = event else {
            return HandleResult::Ignored;
        };
        match *id {
            CM_SIDEKICK_SHOW => {
                let Some(show) = data.as_ref().and_then(|d| d.downcast_ref::<SidekickRequest>()) else {
                    return HandleResult::Ignored;
                };
                let h = show.rect.h();
                let w = show.rect.w();
                let emitter = show.emitter_id;
                let Some(view) = show.take_view() else {
                    return HandleResult::Ignored;
                };
                if self.group.child_count() > 0 {
                    self.group.remove(0);
                }
                self.group.insert(view);
                self.group.set_child_bounds(0, Rect::new(0, 0, w, h));
                self.group.mark_dirty();
                self.request_reposition(w, h, 0, -(h as i16), Some(emitter));
                HandleResult::Consumed
            }
            CM_SIDEKICK_HIDE => {
                if self.group.child_count() > 0 {
                    self.group.remove(0);
                }
                self.group.mark_dirty();
                self.request_reposition(0, 0, 0, 0, None);
                HandleResult::Consumed
            }
            CM_SIDEKICK_NEXT | CM_SIDEKICK_PREV | CM_SIDEKICK_APPLY => {
                // Forward to child
                if self.group.child_count() > 0 {
                    self.group.dispatch(event);
                }
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
