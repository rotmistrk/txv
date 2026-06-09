//! StatusBar — a Group with priority-based horizontal layout.
//!
//! Children are Views that draw and handle their own keys.
//! Layout assigns bounds based on priority, gravity, min/max size, and stretch.
//! Items that don't fit (lowest priority) get zero-width bounds (hidden).

use crate::geometry::Rect;
use crate::group::GroupState;
use crate::palette::{palette, StyleId};
use crate::view::{View, ViewOptions};

use super::gravity::Gravity;
use super::hints::Hints;
use super::status_slot::StatusSlot;

/// StatusBar group container.
pub struct StatusBar {
    group: GroupState,
    hints: Vec<Hints>,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            group: GroupState::new(ViewOptions {
                preprocess: true,
                focusable: false,
                ..ViewOptions::default()
            }),
            hints: Vec::new(),
        }
    }

    /// Add a child view with layout hints from the slot builder.
    pub fn add(&mut self, slot: StatusSlot) {
        let (view, priority, min_width, max_width, stretch, gravity) = slot.take_view();
        let initial_w = view.bounds().w;
        self.group.insert(view);
        self.hints.push(Hints {
            priority,
            min_width,
            max_width,
            stretch,
            gravity,
            natural_width: initial_w,
            last_alloc: 0,
        });
    }

    /// Remove all children.
    pub fn clear(&mut self) {
        while self.group.child_count() > 0 {
            self.group.remove(0);
        }
        self.hints.clear();
    }

    /// Number of items.
    pub fn item_count(&self) -> usize {
        self.group.child_count()
    }

    // --- Internal accessors for layout module ---

    pub(super) fn bounds_rect(&self) -> Rect {
        self.group.bounds()
    }

    pub(super) fn hint_iter(&self) -> impl Iterator<Item = (u8, u16, u16, u16, Gravity, u16, u16)> + '_ {
        self.hints.iter().map(|h| {
            (
                h.priority,
                h.min_width,
                h.max_width,
                h.stretch,
                h.gravity,
                h.natural_width,
                h.last_alloc,
            )
        })
    }

    pub(super) fn child_buffer_width(&self, idx: usize) -> u16 {
        self.group.child(idx).map_or(0, |c| c.bounds().w)
    }

    pub(super) fn child_desired_width(&self, idx: usize) -> u16 {
        self.group.child(idx).map_or(0, |c| c.desired_width())
    }

    pub(super) fn child_count(&self) -> usize {
        self.group.child_count()
    }

    pub(super) fn set_child_rect(&mut self, idx: usize, rect: Rect) {
        self.group.set_child_bounds(idx, rect);
    }

    pub(super) fn set_last_alloc(&mut self, idx: usize, alloc: u16) {
        if let Some(h) = self.hints.get_mut(idx) {
            h.last_alloc = alloc;
        }
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl View for StatusBar {
    crate::delegate_group_state!(group, override { set_bounds, select, unselect });

    fn set_bounds(&mut self, rect: Rect) {
        self.group.set_bounds(rect);
        self.recompute_layout();
        self.group.mark_dirty();
    }

    fn select(&mut self) {}
    fn unselect(&mut self) {}

    fn handle(&mut self, event: &crate::event::Event) -> crate::view::HandleResult {
        let result = self.group.dispatch(event);
        // Re-layout after dispatch: children may have changed desired_width
        self.recompute_layout();
        result
    }

    fn draw(&mut self) {
        let bounds = self.group.bounds();
        if bounds.w == 0 || bounds.h == 0 {
            return;
        }
        let bar_style = palette().style(StyleId::StatusBar);
        self.group.buffer_mut().fill(' ', bar_style);
    }
}
