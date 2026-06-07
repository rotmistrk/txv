//! Dialog — a modal window (framed, centered, captures all input).

use crate::view::ViewOptions;
use crate::window::WindowState;

/// Common dialog state — embed in modal dialog views.
pub struct DialogState {
    pub(crate) window: WindowState,
}

impl DialogState {
    pub fn new() -> Self {
        Self {
            window: WindowState::new(ViewOptions::default().with_focusable().with_modal()),
        }
    }

    pub fn window(&self) -> &WindowState {
        &self.window
    }

    pub fn window_mut(&mut self) -> &mut WindowState {
        &mut self.window
    }
}

impl Default for DialogState {
    fn default() -> Self {
        Self::new()
    }
}

/// Delegates View trait methods for a dialog (via WindowState → GroupState → ViewState).
///
/// Usage: `delegate_dialog_state!(dialog);` inside `impl View for MyDialog { ... }`
/// You still implement `draw()` and `handle()` yourself.
///
/// Override usage (same as delegate_view_state!):
/// ```rust,ignore
/// delegate_dialog_state!(dialog, override { set_bounds });
/// ```
#[macro_export]
macro_rules! delegate_dialog_state {
    ($field:ident) => {
        fn bounds(&self) -> $crate::geometry::Rect {
            self.$field.window.group.bounds()
        }
        fn set_bounds(&mut self, r: $crate::geometry::Rect) {
            self.$field.window.group.set_bounds(r);
        }
        fn options(&self) -> $crate::view::ViewOptions {
            self.$field.window.group.options()
        }
        fn title(&self) -> &str {
            self.$field.window.group.title()
        }
        fn needs_redraw(&self) -> bool {
            self.$field.window.group.any_dirty()
        }
        fn mark_redrawn(&mut self) {
            self.$field.window.group.mark_redrawn();
            for i in 0..self.$field.window.group.child_count() {
                if let Some(child) = self.$field.window.group.child_mut(i) { child.mark_redrawn(); }
            }
        }
        fn select(&mut self) {
            self.$field.window.group.set_focused(true);
            self.$field.window.group.mark_dirty();
            if let Some(child) = self.$field.window.group.focused_child_mut() {
                child.select();
            }
        }
        fn unselect(&mut self) {
            self.$field.window.group.set_focused(false);
            self.$field.window.group.mark_dirty();
            if let Some(child) = self.$field.window.group.focused_child_mut() {
                child.unselect();
            }
        }
    };
    ($field:ident, override { $($skip:ident),* $(,)? }) => {
        $crate::__dvs_maybe!(bounds, [$($skip),*], {
            fn bounds(&self) -> $crate::geometry::Rect {
                self.$field.window.group.bounds()
            }
        });
        $crate::__dvs_maybe!(set_bounds, [$($skip),*], {
            fn set_bounds(&mut self, r: $crate::geometry::Rect) {
                self.$field.window.group.set_bounds(r);
            }
        });
        $crate::__dvs_maybe!(options, [$($skip),*], {
            fn options(&self) -> $crate::view::ViewOptions {
                self.$field.window.group.options()
            }
        });
        $crate::__dvs_maybe!(title, [$($skip),*], {
            fn title(&self) -> &str {
                self.$field.window.group.title()
            }
        });
        $crate::__dvs_maybe!(needs_redraw, [$($skip),*], {
            fn needs_redraw(&self) -> bool {
                self.$field.window.group.any_dirty()
            }
        });
        $crate::__dvs_maybe!(mark_redrawn, [$($skip),*], {
            fn mark_redrawn(&mut self) {
                self.$field.window.group.mark_redrawn();
                for i in 0..self.$field.window.group.child_count() {
                    if let Some(child) = self.$field.window.group.child_mut(i) { child.mark_redrawn(); }
                }
            }
        });
        $crate::__dvs_maybe!(select, [$($skip),*], {
            fn select(&mut self) {
                self.$field.window.group.set_focused(true);
                self.$field.window.group.mark_dirty();
                if let Some(child) = self.$field.window.group.focused_child_mut() {
                    child.select();
                }
            }
        });
        $crate::__dvs_maybe!(unselect, [$($skip),*], {
            fn unselect(&mut self) {
                self.$field.window.group.set_focused(false);
                self.$field.window.group.mark_dirty();
                if let Some(child) = self.$field.window.group.focused_child_mut() {
                    child.unselect();
                }
            }
        });
    };
}
