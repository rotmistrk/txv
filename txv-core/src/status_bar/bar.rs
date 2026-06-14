//! StatusBar — a Group with priority-based horizontal layout.
//!
//! Children self-size via their bounds().w(). Layout assigns positions
//! based on priority, gravity, and stretch.
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
        let (view, priority, stretch, gravity) = slot.take_view();
        let natural_width = view.bounds().w();
        self.group.insert(view);
        self.hints.push(Hints {
            priority,
            stretch,
            gravity,
            natural_width,
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

    /// Collect key binding descriptions from all children.
    pub fn describe_bindings(&self) -> Vec<crate::key_help::KeyHelpEntry> {
        let mut entries = Vec::new();
        for i in 0..self.group.child_count() {
            if let Some(child) = self.group.child(i) {
                entries.extend(child.key_help());
            }
        }
        entries
    }

    // --- Internal accessors for layout module ---

    pub(super) fn bounds_rect(&self) -> Rect {
        self.group.bounds()
    }

    pub(super) fn hint_iter(&self) -> impl Iterator<Item = (u8, u16, Gravity, u16)> + '_ {
        self.hints
            .iter()
            .map(|h| (h.priority, h.stretch, h.gravity, h.natural_width))
    }

    /// Get a child's wanted width for layout purposes.
    /// Stretch items: use natural width only (stretch fills extra space separately).
    /// Non-stretch items that self-resize: max of natural and current.
    pub(super) fn child_wanted_width(&self, idx: usize) -> u16 {
        let h = match self.hints.get(idx) {
            Some(h) => h,
            None => return 0,
        };
        if h.stretch > 0 {
            // For stretch items with natural_width=0 (e.g. dormant FocusGatedGroup),
            // check live bounds — they may have activated since add-time.
            if h.natural_width == 0 {
                return self.group.child(idx).map_or(0, |c| c.bounds().w());
            }
            return h.natural_width;
        }
        let current = self.group.child(idx).map_or(0, |c| c.bounds().w());
        if current > h.natural_width {
            current
        } else {
            h.natural_width
        }
    }

    pub(super) fn child_count(&self) -> usize {
        self.group.child_count()
    }

    pub(super) fn set_child_rect(&mut self, idx: usize, rect: Rect) {
        self.group.set_child_bounds(idx, rect);
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

    fn key_help(&self) -> Vec<crate::key_help::KeyHelpEntry> {
        self.describe_bindings()
    }

    fn handle(&mut self, event: &crate::event::Event) -> crate::view::HandleResult {
        let result = self.group.dispatch(event);
        self.recompute_layout();
        result
    }

    fn draw(&mut self) {
        let bounds = self.group.bounds();
        if bounds.w() == 0 || bounds.h() == 0 {
            return;
        }
        let bar_style = palette().style(StyleId::StatusBar);
        self.group.buffer_mut().fill(' ', bar_style);
    }
}
