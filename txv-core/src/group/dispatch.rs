//! Three-phase event dispatch and delegate_group_state! macro.

use super::GroupState;
use crate::commands::{RepositionRequest, CM_REPOSITION};
use crate::event::Event;
use crate::geometry::Rect;
use crate::view::{EventSink, HandleResult};

impl GroupState {
    /// Three-phase event dispatch.
    pub fn dispatch(&mut self, event: &Event) -> HandleResult {
        // CM_REPOSITION: the group itself handles it (child asks parent to move it).
        if let Event::Command { id, data, .. } = event {
            if *id == CM_REPOSITION {
                return self.handle_reposition(data);
            }
        }

        // Broadcast commands: deliver to ALL children.
        if matches!(event, Event::Command { broadcast: true, .. }) {
            for child in &mut self.children {
                child.handle(event);
            }
            return HandleResult::Ignored;
        }

        // Phase 1: preprocess
        for child in &mut self.children {
            if child.options().preprocess && child.handle(event) == HandleResult::Consumed {
                return HandleResult::Consumed;
            }
        }

        // Phase 2: modal child or focused child
        let target = self.modal_child().unwrap_or(self.focused);
        if let Some(child) = self.children.get_mut(target) {
            if child.handle(event) == HandleResult::Consumed {
                return HandleResult::Consumed;
            }
        }

        // Phase 3: postprocess
        for child in &mut self.children {
            if child.options().postprocess && child.handle(event) == HandleResult::Consumed {
                return HandleResult::Consumed;
            }
        }

        HandleResult::Ignored
    }

    fn handle_reposition(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        let Some(req) = data.as_ref().and_then(|d| d.downcast_ref::<RepositionRequest>()) else {
            return HandleResult::Ignored;
        };
        let base = match req.relative_to {
            Some(rel_id) => self.origin_of(rel_id).unwrap_or((0, 0)),
            None => (0, 0),
        };
        let x = (base.0 as i16 + req.offset_x).max(0) as u16;
        let y = (base.1 as i16 + req.offset_y).max(0) as u16;
        let translated = Rect::new(x, y, req.width, req.height);
        for i in 0..self.children.len() {
            if self.children[i].view_id() == req.view_id {
                self.set_child_bounds(i, translated);
                self.mark_dirty();
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }

    /// Returns true if any child needs redraw.
    pub fn any_dirty(&self) -> bool {
        self.view.is_dirty() || self.children.iter().any(|c| c.needs_redraw())
    }

    /// Set the event sink on this group and propagate to all children.
    pub fn set_sink(&mut self, sink: EventSink) {
        self.view.set_sink(sink.clone());
        for child in &mut self.children {
            child.set_sink(sink.clone());
        }
    }

    /// Propagate the current sink to a newly inserted child.
    pub(super) fn propagate_sink_to(&mut self, index: usize) {
        if let Some(sink) = self.view.sink().cloned() {
            if let Some(child) = self.children.get_mut(index) {
                child.set_sink(sink);
            }
        }
    }

    fn modal_child(&self) -> Option<usize> {
        self.children.iter().position(|c| c.options().modal)
    }
}

/// Delegates View trait methods for a group (ViewState via GroupState forwarding methods).
#[macro_export]
macro_rules! delegate_group_state {
    ($field:ident) => {
        fn view_id(&self) -> $crate::view::ViewId { self.$field.view_id() }
        fn bounds(&self) -> $crate::geometry::Rect { self.$field.bounds() }
        fn set_bounds(&mut self, r: $crate::geometry::Rect) { self.$field.set_bounds(r); }
        fn set_sink(&mut self, sink: $crate::view::EventSink) { self.$field.set_sink(sink); }
        fn options(&self) -> $crate::view::ViewOptions { self.$field.options() }
        fn title(&self) -> &str { self.$field.title() }
        fn needs_redraw(&self) -> bool { self.$field.any_dirty() }
        fn mark_redrawn(&mut self) {
            self.$field.mark_redrawn();
            for i in 0..self.$field.child_count() {
                if let Some(child) = self.$field.child_mut(i) { child.mark_redrawn(); }
            }
        }
        fn select(&mut self) {
            self.$field.set_focused(true); self.$field.mark_dirty();
            if let Some(child) = self.$field.focused_child_mut() { child.select(); }
        }
        fn unselect(&mut self) {
            self.$field.set_focused(false); self.$field.mark_dirty();
            if let Some(child) = self.$field.focused_child_mut() { child.unselect(); }
        }
        fn cursor(&self) -> Option<$crate::cursor::CursorRequest> {
            self.$field.cursor()
        }
        fn buffer(&self) -> &$crate::buffer::Buffer {
            self.$field.buffer()
        }
        fn group_state(&self) -> Option<&$crate::group::GroupState> {
            Some(&self.$field)
        }
        fn render(&mut self) -> bool {
            let own_dirty = self.$field.is_dirty();
            let mut child_drew = false;
            for i in 0..self.$field.child_count() {
                if !self.$field.is_child_visible(i) {
                    continue;
                }
                if let Some(child) = self.$field.child_mut(i) {
                    child_drew |= child.render();
                }
            }
            if own_dirty || child_drew {
                self.draw();
                self.$field.mark_redrawn();
                for i in 0..self.$field.child_count() {
                    if !self.$field.is_child_visible(i) {
                        continue;
                    }
                    self.$field.blit_child(i);
                }
                return true;
            }
            false
        }
    };
    ($field:ident, override { $($skip:ident),* $(,)? }) => {
        $crate::__dvs_maybe!(view_id, [$($skip),*], {
            fn view_id(&self) -> $crate::view::ViewId { self.$field.view_id() }
        });
        $crate::__dvs_maybe!(bounds, [$($skip),*], {
            fn bounds(&self) -> $crate::geometry::Rect { self.$field.bounds() }
        });
        $crate::__dvs_maybe!(set_bounds, [$($skip),*], {
            fn set_bounds(&mut self, r: $crate::geometry::Rect) { self.$field.set_bounds(r); }
        });
        $crate::__dvs_maybe!(set_sink, [$($skip),*], {
            fn set_sink(&mut self, sink: $crate::view::EventSink) { self.$field.set_sink(sink); }
        });
        $crate::__dvs_maybe!(options, [$($skip),*], {
            fn options(&self) -> $crate::view::ViewOptions { self.$field.options() }
        });
        $crate::__dvs_maybe!(title, [$($skip),*], {
            fn title(&self) -> &str { self.$field.title() }
        });
        $crate::__dvs_maybe!(needs_redraw, [$($skip),*], {
            fn needs_redraw(&self) -> bool { self.$field.any_dirty() }
        });
        $crate::__dvs_maybe!(mark_redrawn, [$($skip),*], {
            fn mark_redrawn(&mut self) {
                self.$field.mark_redrawn();
                for i in 0..self.$field.child_count() {
                    if let Some(child) = self.$field.child_mut(i) { child.mark_redrawn(); }
                }
            }
        });
        $crate::__dvs_maybe!(select, [$($skip),*], {
            fn select(&mut self) {
                self.$field.set_focused(true); self.$field.mark_dirty();
                if let Some(child) = self.$field.focused_child_mut() { child.select(); }
            }
        });
        $crate::__dvs_maybe!(unselect, [$($skip),*], {
            fn unselect(&mut self) {
                self.$field.set_focused(false); self.$field.mark_dirty();
                if let Some(child) = self.$field.focused_child_mut() { child.unselect(); }
            }
        });
        $crate::__dvs_maybe!(cursor, [$($skip),*], {
            fn cursor(&self) -> Option<$crate::cursor::CursorRequest> {
                self.$field.cursor()
            }
        });
        $crate::__dvs_maybe!(buffer, [$($skip),*], {
            fn buffer(&self) -> &$crate::buffer::Buffer {
                self.$field.buffer()
            }
        });
        fn group_state(&self) -> Option<&$crate::group::GroupState> {
            Some(&self.$field)
        }
        fn render(&mut self) -> bool {
            let own_dirty = self.$field.is_dirty();
            let mut child_drew = false;
            for i in 0..self.$field.child_count() {
                if !self.$field.is_child_visible(i) {
                    continue;
                }
                if let Some(child) = self.$field.child_mut(i) {
                    child_drew |= child.render();
                }
            }
            if own_dirty || child_drew {
                self.draw();
                self.$field.mark_redrawn();
                for i in 0..self.$field.child_count() {
                    if !self.$field.is_child_visible(i) {
                        continue;
                    }
                    self.$field.blit_child(i);
                }
                return true;
            }
            false
        }
    };
}
