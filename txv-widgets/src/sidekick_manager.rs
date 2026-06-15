//! SidekickManager — postprocess group that hosts a popup View.
//!
//! On CM_SIDEKICK_SHOW: takes ownership of view as child 0, emits CM_REPOSITION.
//! On CM_SIDEKICK_HIDE: removes child 0, repositions to 0×0.
//! On CM_DROPDOWN_DONE: extracts selected text, emits CM_SIDEKICK_RESULT, hides.
//! On CM_DROPDOWN_CANCELLED: hides.

use txv_core::commands::{RepositionRequest, CM_REPOSITION};
use txv_core::prelude::*;

use crate::dropdown_menu::{DropdownMenu, CM_DROPDOWN_CANCELLED, CM_DROPDOWN_DONE};
use crate::input_line::completion_source::CompletionSource;
use crate::sidekick::{SidekickRequest, CM_SIDEKICK_HIDE, CM_SIDEKICK_RESULT, CM_SIDEKICK_SHOW};

pub struct SidekickManager {
    group: GroupState,
}

impl SidekickManager {
    pub fn new() -> Self {
        Self {
            group: GroupState::new(ViewOptions::default().with_postprocess()),
        }
    }

    fn hide(&mut self) {
        if self.group.child_count() > 0 {
            self.group.remove(0);
        }
        self.group.mark_dirty();
        self.request_reposition(0, 0, 0, 0, None);
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
            CM_SIDEKICK_SHOW => self.handle_show(data),
            CM_SIDEKICK_HIDE => {
                self.hide();
                HandleResult::Consumed
            }
            CM_DROPDOWN_DONE => {
                self.handle_done(data);
                HandleResult::Consumed
            }
            CM_DROPDOWN_CANCELLED => {
                self.hide();
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl SidekickManager {
    fn handle_show(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
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
        let (off_x, off_y) = if show.rect.x() == 0 && show.rect.y() == 0 {
            (0i16, -(h as i16))
        } else {
            let cx = show.rect.x() as i16;
            let cy = show.rect.y() as i16;
            (cx, cy + 1)
        };
        self.request_reposition(w, h, off_x, off_y, Some(emitter));
        HandleResult::Consumed
    }

    fn handle_done(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) {
        let idx = data
            .as_ref()
            .and_then(|d| d.downcast_ref::<usize>())
            .copied()
            .unwrap_or(0);
        let text = self
            .group
            .child_mut(0)
            .and_then(|c| c.as_any_mut())
            .and_then(|a| a.downcast_mut::<DropdownMenu<CompletionSource>>())
            .and_then(|dd| dd.source().text_at(idx).map(String::from));
        if let Some(t) = text {
            self.group.put_command(CM_SIDEKICK_RESULT, Some(Box::new(t)));
        }
        self.hide();
    }
}

impl Default for SidekickManager {
    fn default() -> Self {
        Self::new()
    }
}
