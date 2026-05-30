//! StatusBar — a Group with priority-based horizontal layout.
//!
//! Children are Views that draw and handle their own keys.
//! Layout assigns bounds based on priority, gravity, min/max size, and stretch.
//! Items that don't fit (lowest priority) get zero-width bounds (hidden).

use crate::buffer::Buffer;

use crate::event::Event;
use crate::geometry::Rect;
use crate::group::GroupState;
use crate::view::{EventSink, HandleResult, View, ViewOptions};

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

    pub(super) fn hint_iter(&self) -> impl Iterator<Item = (u8, u16, u16, u16, Gravity, u16)> + '_ {
        self.hints.iter().map(|h| {
            (
                h.priority,
                h.min_width,
                h.max_width,
                h.stretch,
                h.gravity,
                h.natural_width,
            )
        })
    }

    pub(super) fn child_buffer_width(&self, idx: usize) -> u16 {
        self.group.child(idx).map_or(0, |c| c.bounds().w)
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
    fn bounds(&self) -> Rect {
        self.group.bounds()
    }

    fn set_bounds(&mut self, rect: Rect) {
        self.group.set_bounds(rect);
        self.recompute_layout();
        self.group.mark_dirty();
    }

    fn set_sink(&mut self, sink: EventSink) {
        self.group.set_sink(sink);
    }

    fn options(&self) -> ViewOptions {
        self.group.options()
    }

    fn title(&self) -> &str {
        ""
    }

    fn needs_redraw(&self) -> bool {
        self.group.any_dirty()
    }

    fn mark_redrawn(&mut self) {
        self.group.mark_redrawn();
        for i in 0..self.group.child_count() {
            if let Some(child) = self.group.child_mut(i) {
                child.mark_redrawn();
            }
        }
    }

    fn select(&mut self) {}
    fn unselect(&mut self) {}

    fn draw(&mut self) {
        let bounds = self.group.bounds();
        if bounds.w == 0 || bounds.h == 0 {
            return;
        }
        self.recompute_layout();

        let bar_style = crate::palette::palette().style(crate::palette::StyleId::StatusBar);
        self.group.buffer_mut().fill(' ', bar_style);

        // Draw children into their buffers
        for i in 0..self.group.child_count() {
            if let Some(child) = self.group.child_mut(i) {
                if child.bounds().w > 0 {
                    child.draw();
                }
            }
        }

        // Blit child buffers into group buffer
        let buf_ptr = self.group.buffer_mut() as *mut Buffer;
        for i in 0..self.group.child_count() {
            if let Some(child) = self.group.child(i) {
                if child.bounds().w == 0 {
                    continue;
                }
                let (ox, oy) = self.group.child_origin(i);
                unsafe { (*buf_ptr).blit(child.buffer(), ox, oy) };
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        self.group.dispatch(event)
    }

    fn cursor(&self) -> Option<crate::cursor::CursorRequest> {
        self.group.cursor()
    }

    fn buffer(&self) -> &Buffer {
        self.group.buffer()
    }

    fn group_state(&self) -> Option<&crate::group::GroupState> {
        Some(&self.group)
    }
}
