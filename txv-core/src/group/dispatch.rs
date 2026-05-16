//! Three-phase event dispatch and delegate_group_state! macro.

use super::GroupState;
use crate::event::Event;
use crate::view::{EventSink, HandleResult};

impl GroupState {
    /// Three-phase event dispatch.
    pub fn dispatch(&mut self, event: &Event) -> HandleResult {
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

/// Delegates View trait methods for a group (ViewState via `$field.view` + group methods).
#[macro_export]
macro_rules! delegate_group_state {
    ($field:ident) => {
        fn bounds(&self) -> $crate::geometry::Rect { self.$field.view.bounds() }
        fn set_bounds(&mut self, r: $crate::geometry::Rect) { self.$field.view.set_bounds(r); }
        fn set_sink(&mut self, sink: $crate::view::EventSink) { self.$field.set_sink(sink); }
        fn options(&self) -> $crate::view::ViewOptions { self.$field.view.options() }
        fn title(&self) -> &str { self.$field.view.title() }
        fn needs_redraw(&self) -> bool { self.$field.any_dirty() }
        fn mark_redrawn(&mut self) {
            self.$field.view.mark_redrawn();
            for i in 0..self.$field.child_count() {
                if let Some(child) = self.$field.child_mut(i) { child.mark_redrawn(); }
            }
        }
        fn select(&mut self) {
            self.$field.view.set_focused(true); self.$field.view.mark_dirty();
            if let Some(child) = self.$field.focused_child_mut() { child.select(); }
        }
        fn unselect(&mut self) {
            self.$field.view.set_focused(false); self.$field.view.mark_dirty();
            if let Some(child) = self.$field.focused_child_mut() { child.unselect(); }
        }
        fn buffer(&self) -> &$crate::buffer::Buffer {
            self.$field.view.buffer()
        }
    };
    ($field:ident, override { $($skip:ident),* $(,)? }) => {
        $crate::__dvs_maybe!(bounds, [$($skip),*], { fn bounds(&self) -> $crate::geometry::Rect { self.$field.view.bounds() } });
        $crate::__dvs_maybe!(set_bounds, [$($skip),*], { fn set_bounds(&mut self, r: $crate::geometry::Rect) { self.$field.view.set_bounds(r); } });
        $crate::__dvs_maybe!(set_sink, [$($skip),*], { fn set_sink(&mut self, sink: $crate::view::EventSink) { self.$field.set_sink(sink); } });
        $crate::__dvs_maybe!(options, [$($skip),*], { fn options(&self) -> $crate::view::ViewOptions { self.$field.view.options() } });
        $crate::__dvs_maybe!(title, [$($skip),*], { fn title(&self) -> &str { self.$field.view.title() } });
        $crate::__dvs_maybe!(needs_redraw, [$($skip),*], { fn needs_redraw(&self) -> bool { self.$field.any_dirty() } });
        $crate::__dvs_maybe!(mark_redrawn, [$($skip),*], {
            fn mark_redrawn(&mut self) {
                self.$field.view.mark_redrawn();
                for i in 0..self.$field.child_count() {
                    if let Some(child) = self.$field.child_mut(i) { child.mark_redrawn(); }
                }
            }
        });
        $crate::__dvs_maybe!(select, [$($skip),*], {
            fn select(&mut self) {
                self.$field.view.set_focused(true); self.$field.view.mark_dirty();
                if let Some(child) = self.$field.focused_child_mut() { child.select(); }
            }
        });
        $crate::__dvs_maybe!(unselect, [$($skip),*], {
            fn unselect(&mut self) {
                self.$field.view.set_focused(false); self.$field.view.mark_dirty();
                if let Some(child) = self.$field.focused_child_mut() { child.unselect(); }
            }
        });
        $crate::__dvs_maybe!(buffer, [$($skip),*], {
            fn buffer(&self) -> &$crate::buffer::Buffer {
                self.$field.view.buffer()
            }
        });
    };
}
