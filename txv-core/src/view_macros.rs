//! View delegation macros — delegate_view_state!, delegate_view!, and helpers.

/// Delegates View trait boilerplate to a `ViewState` field.
///
/// # Basic usage
///
/// ```rust,ignore
/// delegate_view_state!(state);
/// ```
///
/// # Override usage
///
/// ```rust,ignore
/// delegate_view_state!(state, override { set_bounds, select, unselect });
/// ```
#[macro_export]
macro_rules! delegate_view_state {
    ($field:ident) => {
        fn view_id(&self) -> $crate::view::ViewId {
            self.$field.id()
        }
        fn bounds(&self) -> $crate::geometry::Rect {
            self.$field.bounds()
        }
        fn set_bounds(&mut self, r: $crate::geometry::Rect) {
            self.$field.set_bounds(r);
        }
        fn set_sink(&mut self, sink: $crate::view::EventSink) {
            self.$field.set_sink(sink);
        }
        fn options(&self) -> $crate::view::ViewOptions {
            self.$field.options()
        }
        fn title(&self) -> &str {
            self.$field.title()
        }
        fn needs_redraw(&self) -> bool {
            self.$field.is_dirty()
        }
        fn mark_redrawn(&mut self) {
            self.$field.mark_redrawn();
        }
        fn select(&mut self) {
            self.$field.set_focused(true);
            self.$field.mark_dirty();
        }
        fn unselect(&mut self) {
            self.$field.set_focused(false);
            self.$field.mark_dirty();
        }
        fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
            Some(self)
        }
        fn buffer(&self) -> &$crate::buffer::Buffer {
            self.$field.buffer()
        }
    };
    ($field:ident, override { $($skip:ident),* $(,)? }) => {
        $crate::__dvs_maybe!(view_id, [$($skip),*], {
            fn view_id(&self) -> $crate::view::ViewId {
                self.$field.id()
            }
        });
        $crate::__dvs_maybe!(bounds, [$($skip),*], {
            fn bounds(&self) -> $crate::geometry::Rect {
                self.$field.bounds()
            }
        });
        $crate::__dvs_maybe!(set_bounds, [$($skip),*], {
            fn set_bounds(&mut self, r: $crate::geometry::Rect) {
                self.$field.set_bounds(r);
            }
        });
        $crate::__dvs_maybe!(set_sink, [$($skip),*], {
            fn set_sink(&mut self, sink: $crate::view::EventSink) {
                self.$field.set_sink(sink);
            }
        });
        $crate::__dvs_maybe!(options, [$($skip),*], {
            fn options(&self) -> $crate::view::ViewOptions {
                self.$field.options()
            }
        });
        $crate::__dvs_maybe!(title, [$($skip),*], {
            fn title(&self) -> &str {
                self.$field.title()
            }
        });
        $crate::__dvs_maybe!(needs_redraw, [$($skip),*], {
            fn needs_redraw(&self) -> bool {
                self.$field.is_dirty()
            }
        });
        $crate::__dvs_maybe!(mark_redrawn, [$($skip),*], {
            fn mark_redrawn(&mut self) {
                self.$field.mark_redrawn();
            }
        });
        $crate::__dvs_maybe!(select, [$($skip),*], {
            fn select(&mut self) {
                self.$field.set_focused(true);
                self.$field.mark_dirty();
            }
        });
        $crate::__dvs_maybe!(unselect, [$($skip),*], {
            fn unselect(&mut self) {
                self.$field.set_focused(false);
                self.$field.mark_dirty();
            }
        });
        $crate::__dvs_maybe!(as_any_mut, [$($skip),*], {
            fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
                Some(self)
            }
        });
        $crate::__dvs_maybe!(buffer, [$($skip),*], {
            fn buffer(&self) -> &$crate::buffer::Buffer {
                self.$field.buffer()
            }
        });
        $crate::__dvs_maybe!(cursor, [$($skip),*], {
            fn cursor(&self) -> Option<$crate::cursor::CursorRequest> { None }
        });
    };
}

/// Internal: emit `$body` only if `$method` is NOT in the skip list.
#[macro_export]
#[doc(hidden)]
macro_rules! __dvs_maybe {
    (bounds, [bounds $(, $rest:ident)*], { $($body:tt)* }) => {};
    (set_bounds, [set_bounds $(, $rest:ident)*], { $($body:tt)* }) => {};
    (set_sink, [set_sink $(, $rest:ident)*], { $($body:tt)* }) => {};
    (options, [options $(, $rest:ident)*], { $($body:tt)* }) => {};
    (view_id, [view_id $(, $rest:ident)*], { $($body:tt)* }) => {};
    (title, [title $(, $rest:ident)*], { $($body:tt)* }) => {};
    (needs_redraw, [needs_redraw $(, $rest:ident)*], { $($body:tt)* }) => {};
    (mark_redrawn, [mark_redrawn $(, $rest:ident)*], { $($body:tt)* }) => {};
    (select, [select $(, $rest:ident)*], { $($body:tt)* }) => {};
    (unselect, [unselect $(, $rest:ident)*], { $($body:tt)* }) => {};
    (buffer, [buffer $(, $rest:ident)*], { $($body:tt)* }) => {};
    (cursor, [cursor $(, $rest:ident)*], { $($body:tt)* }) => {};
    (as_any_mut, [as_any_mut $(, $rest:ident)*], { $($body:tt)* }) => {};
    ($method:ident, [$head:ident $(, $rest:ident)*], { $($body:tt)* }) => {
        $crate::__dvs_maybe!($method, [$($rest),*], { $($body)* });
    };
    ($method:ident, [], { $($body:tt)* }) => {
        $($body)*
    };
}

/// Delegates View trait methods to an inner View field (wrapper pattern).
#[macro_export]
macro_rules! delegate_view {
    ($field:ident, override { $($skip:ident),* $(,)? }) => {
        $crate::__dv_maybe!(bounds, [$($skip),*], {
            fn bounds(&self) -> $crate::geometry::Rect { self.$field.bounds() }
        });
        $crate::__dv_maybe!(set_bounds, [$($skip),*], {
            fn set_bounds(&mut self, r: $crate::geometry::Rect) { self.$field.set_bounds(r); }
        });
        $crate::__dv_maybe!(set_sink, [$($skip),*], {
            fn set_sink(&mut self, sink: $crate::view::EventSink) { self.$field.set_sink(sink); }
        });
        $crate::__dv_maybe!(options, [$($skip),*], {
            fn options(&self) -> $crate::view::ViewOptions { self.$field.options() }
        });
        $crate::__dv_maybe!(title, [$($skip),*], {
            fn title(&self) -> &str { self.$field.title() }
        });
        $crate::__dv_maybe!(needs_redraw, [$($skip),*], {
            fn needs_redraw(&self) -> bool { self.$field.needs_redraw() }
        });
        $crate::__dv_maybe!(mark_redrawn, [$($skip),*], {
            fn mark_redrawn(&mut self) { self.$field.mark_redrawn(); }
        });
        $crate::__dv_maybe!(select, [$($skip),*], {
            fn select(&mut self) { self.$field.select(); }
        });
        $crate::__dv_maybe!(unselect, [$($skip),*], {
            fn unselect(&mut self) { self.$field.unselect(); }
        });
        $crate::__dv_maybe!(draw, [$($skip),*], {
            fn draw(&mut self) { self.$field.draw(); }
        });
        $crate::__dv_maybe!(handle, [$($skip),*], {
            fn handle(&mut self, event: &$crate::event::Event) -> $crate::view::HandleResult {
                self.$field.handle(event)
            }
        });
        $crate::__dv_maybe!(buffer, [$($skip),*], {
            fn buffer(&self) -> &$crate::buffer::Buffer { self.$field.buffer() }
        });
        $crate::__dv_maybe!(cursor, [$($skip),*], {
            fn cursor(&self) -> Option<$crate::cursor::CursorRequest> { self.$field.cursor() }
        });
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __dv_maybe {
    (bounds, [bounds $(, $rest:ident)*], { $($body:tt)* }) => {};
    (set_bounds, [set_bounds $(, $rest:ident)*], { $($body:tt)* }) => {};
    (set_sink, [set_sink $(, $rest:ident)*], { $($body:tt)* }) => {};
    (options, [options $(, $rest:ident)*], { $($body:tt)* }) => {};
    (title, [title $(, $rest:ident)*], { $($body:tt)* }) => {};
    (needs_redraw, [needs_redraw $(, $rest:ident)*], { $($body:tt)* }) => {};
    (mark_redrawn, [mark_redrawn $(, $rest:ident)*], { $($body:tt)* }) => {};
    (select, [select $(, $rest:ident)*], { $($body:tt)* }) => {};
    (unselect, [unselect $(, $rest:ident)*], { $($body:tt)* }) => {};
    (draw, [draw $(, $rest:ident)*], { $($body:tt)* }) => {};
    (handle, [handle $(, $rest:ident)*], { $($body:tt)* }) => {};
    (buffer, [buffer $(, $rest:ident)*], { $($body:tt)* }) => {};
    (cursor, [cursor $(, $rest:ident)*], { $($body:tt)* }) => {};
    ($method:ident, [$head:ident $(, $rest:ident)*], { $($body:tt)* }) => {
        $crate::__dv_maybe!($method, [$($rest),*], { $($body)* });
    };
    ($method:ident, [], { $($body:tt)* }) => {
        $($body)*
    };
}
