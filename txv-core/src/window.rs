//! Window — a framed group (title bar, border, optional shadow).

use crate::group::GroupState;
use crate::view::ViewOptions;

/// Frame drawing style.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FrameStyle {
    #[default]
    Single,
    Double,
    None,
}

/// Common window state — embed in framed views.
pub struct WindowState {
    pub(crate) group: GroupState,
    pub(crate) frame: FrameStyle,
    pub(crate) shadow: bool,
}

impl WindowState {
    pub fn new(options: ViewOptions) -> Self {
        Self {
            group: GroupState::new(options),
            frame: FrameStyle::Single,
            shadow: false,
        }
    }

    pub fn group(&self) -> &GroupState {
        &self.group
    }

    pub fn group_mut(&mut self) -> &mut GroupState {
        &mut self.group
    }

    pub fn frame(&self) -> FrameStyle {
        self.frame
    }

    pub fn set_frame(&mut self, frame: FrameStyle) {
        self.frame = frame;
    }

    pub fn shadow(&self) -> bool {
        self.shadow
    }

    pub fn set_shadow(&mut self, shadow: bool) {
        self.shadow = shadow;
    }
}

impl Default for WindowState {
    fn default() -> Self {
        Self::new(ViewOptions::default().with_focusable())
    }
}

/// Delegates View trait methods for a window (via GroupState inside WindowState).
///
/// Usage: `delegate_window_state!(window);` inside `impl View for MyWindow { ... }`
/// You still implement `draw()` and `handle()` yourself.
///
/// Override usage (same as delegate_view_state!):
/// ```rust,ignore
/// delegate_window_state!(window, override { set_bounds });
/// ```
#[macro_export]
macro_rules! delegate_window_state {
    ($field:ident) => {
        fn bounds(&self) -> $crate::geometry::Rect {
            self.$field.group.bounds()
        }
        fn set_bounds(&mut self, r: $crate::geometry::Rect) {
            self.$field.group.set_bounds(r);
        }
        fn options(&self) -> $crate::view::ViewOptions {
            self.$field.group.options()
        }
        fn title(&self) -> &str {
            self.$field.group.title()
        }
        fn needs_redraw(&self) -> bool {
            self.$field.group.any_dirty()
        }
        fn mark_redrawn(&mut self) {
            self.$field.group.mark_redrawn();
            for i in 0..self.$field.group.child_count() {
                if let Some(child) = self.$field.group.child_mut(i) { child.mark_redrawn(); }
            }
        }
        fn select(&mut self) {
            self.$field.group.set_focused(true);
            self.$field.group.mark_dirty();
            if let Some(child) = self.$field.group.focused_child_mut() {
                child.select();
            }
        }
        fn unselect(&mut self) {
            self.$field.group.set_focused(false);
            self.$field.group.mark_dirty();
            if let Some(child) = self.$field.group.focused_child_mut() {
                child.unselect();
            }
        }
    };
    ($field:ident, override { $($skip:ident),* $(,)? }) => {
        $crate::__dvs_maybe!(bounds, [$($skip),*], {
            fn bounds(&self) -> $crate::geometry::Rect {
                self.$field.group.bounds()
            }
        });
        $crate::__dvs_maybe!(set_bounds, [$($skip),*], {
            fn set_bounds(&mut self, r: $crate::geometry::Rect) {
                self.$field.group.set_bounds(r);
            }
        });
        $crate::__dvs_maybe!(options, [$($skip),*], {
            fn options(&self) -> $crate::view::ViewOptions {
                self.$field.group.options()
            }
        });
        $crate::__dvs_maybe!(title, [$($skip),*], {
            fn title(&self) -> &str {
                self.$field.group.title()
            }
        });
        $crate::__dvs_maybe!(needs_redraw, [$($skip),*], {
            fn needs_redraw(&self) -> bool {
                self.$field.group.any_dirty()
            }
        });
        $crate::__dvs_maybe!(mark_redrawn, [$($skip),*], {
            fn mark_redrawn(&mut self) {
                self.$field.group.mark_redrawn();
                for i in 0..self.$field.group.child_count() {
                    if let Some(child) = self.$field.group.child_mut(i) { child.mark_redrawn(); }
                }
            }
        });
        $crate::__dvs_maybe!(select, [$($skip),*], {
            fn select(&mut self) {
                self.$field.group.set_focused(true);
                self.$field.group.mark_dirty();
                if let Some(child) = self.$field.group.focused_child_mut() {
                    child.select();
                }
            }
        });
        $crate::__dvs_maybe!(unselect, [$($skip),*], {
            fn unselect(&mut self) {
                self.$field.group.set_focused(false);
                self.$field.group.mark_dirty();
                if let Some(child) = self.$field.group.focused_child_mut() {
                    child.unselect();
                }
            }
        });
    };
}
