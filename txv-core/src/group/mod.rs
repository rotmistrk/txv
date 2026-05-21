//! Group — a View that owns and dispatches to child Views.
//!
//! Three-phase dispatch:
//! 1. Preprocess: children with `options().preprocess` see event first
//! 2. Focused/modal: the modal child (if any) or focused child handles
//! 3. Postprocess: children with `options().postprocess` see event last

mod dispatch;

use crate::view::{View, ViewOptions, ViewState};

/// Common group state — embed in any view that owns children.
pub struct GroupState {
    view: ViewState,
    pub(crate) children: Vec<Box<dyn View>>,
    pub(crate) focused: usize,
}

impl GroupState {
    pub fn new(options: ViewOptions) -> Self {
        Self {
            view: ViewState::new(options),
            children: Vec::new(),
            focused: 0,
        }
    }

    pub fn insert(&mut self, child: Box<dyn View>) {
        self.children.push(child);
        let idx = self.children.len() - 1;
        self.propagate_sink_to(idx);
        self.view.mark_dirty();
    }

    pub fn insert_at(&mut self, index: usize, child: Box<dyn View>) {
        self.children.insert(index, child);
        self.propagate_sink_to(index);
        self.view.mark_dirty();
    }

    pub fn remove(&mut self, index: usize) -> Box<dyn View> {
        let child = self.children.remove(index);
        if self.focused >= self.children.len() && self.focused > 0 {
            self.focused -= 1;
        }
        self.view.mark_dirty();
        child
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn focused_index(&self) -> usize {
        self.focused
    }

    pub fn set_focused_index(&mut self, index: usize) {
        if index < self.children.len() {
            self.focused = index;
        }
    }

    /// Get immutable reference to a child by index.
    pub fn child(&self, index: usize) -> Option<&dyn View> {
        self.children.get(index).map(|c| c.as_ref())
    }

    /// Get mutable reference to a child by index.
    pub fn child_mut(&mut self, index: usize) -> Option<&mut Box<dyn View>> {
        self.children.get_mut(index)
    }

    /// Get the focused child (immutable).
    pub fn focused_child(&self) -> Option<&dyn View> {
        self.children.get(self.focused).map(|c| c.as_ref())
    }

    /// Get the focused child (mutable).
    pub fn focused_child_mut(&mut self) -> Option<&mut Box<dyn View>> {
        self.children.get_mut(self.focused)
    }

    /// Set bounds on a child by index.
    pub fn set_child_bounds(&mut self, index: usize, rect: crate::geometry::Rect) {
        if let Some(child) = self.children.get_mut(index) {
            child.set_bounds(rect);
        }
    }

    /// Select the focused child, unselect the previous.
    pub fn select_focused(&mut self) {
        if let Some(child) = self.children.get_mut(self.focused) {
            child.select();
        }
    }

    /// Unselect the focused child.
    pub fn unselect_focused(&mut self) {
        if let Some(child) = self.children.get_mut(self.focused) {
            child.unselect();
        }
    }

    /// Switch focus to a new index (unselects old, selects new).
    pub fn switch_focus(&mut self, new_index: usize) {
        if new_index >= self.children.len() || new_index == self.focused {
            return;
        }
        self.children[self.focused].unselect();
        self.focused = new_index;
        self.children[self.focused].select();
        self.view.mark_dirty();
    }

    /// Iterate over children immutably.
    pub fn children_iter(&self) -> impl Iterator<Item = &dyn View> {
        self.children.iter().map(|c| c.as_ref())
    }

    /// Iterate over children mutably.
    pub fn children_iter_mut(&mut self) -> impl Iterator<Item = &mut Box<dyn View>> {
        self.children.iter_mut()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    pub fn focus_next(&mut self) {
        if self.children.is_empty() {
            return;
        }
        let old = self.focused;
        let count = self.children.len();
        let mut next = (old + 1) % count;
        let start = next;
        loop {
            if self.children[next].options().focusable {
                break;
            }
            next = (next + 1) % count;
            if next == start {
                return;
            }
        }
        if old != next {
            self.children[old].unselect();
            self.focused = next;
            self.children[next].select();
            self.view.mark_dirty();
        }
    }

    pub fn focus_prev(&mut self) {
        if self.children.is_empty() {
            return;
        }
        let old = self.focused;
        let count = self.children.len();
        let mut prev = if old == 0 {
            count - 1
        } else {
            old - 1
        };
        let start = prev;
        loop {
            if self.children[prev].options().focusable {
                break;
            }
            prev = if prev == 0 {
                count - 1
            } else {
                prev - 1
            };
            if prev == start {
                return;
            }
        }
        if old != prev {
            self.children[old].unselect();
            self.focused = prev;
            self.children[prev].select();
            self.view.mark_dirty();
        }
    }

    // --- Forwarding methods (delegate to self.view) ---

    pub fn bounds(&self) -> crate::geometry::Rect {
        self.view.bounds()
    }

    pub fn set_bounds(&mut self, r: crate::geometry::Rect) {
        self.view.set_bounds(r);
    }

    pub fn mark_dirty(&mut self) {
        self.view.mark_dirty();
    }

    pub fn mark_redrawn(&mut self) {
        self.view.mark_redrawn();
    }

    pub fn is_dirty(&self) -> bool {
        self.view.is_dirty()
    }

    pub fn is_focused(&self) -> bool {
        self.view.is_focused()
    }

    pub fn set_focused(&mut self, f: bool) {
        self.view.set_focused(f);
    }

    pub fn buffer(&self) -> &crate::buffer::Buffer {
        self.view.buffer()
    }

    pub fn buffer_mut(&mut self) -> &mut crate::buffer::Buffer {
        self.view.buffer_mut()
    }

    pub fn options(&self) -> ViewOptions {
        self.view.options()
    }

    pub fn sink(&self) -> Option<&crate::view::EventSink> {
        self.view.sink()
    }

    pub fn title(&self) -> &str {
        self.view.title()
    }

    pub fn set_title(&mut self, t: impl Into<String>) {
        self.view.set_title(t);
    }

    pub fn put_event(&self, event: crate::event::Event) {
        self.view.put_event(event);
    }

    pub fn put_command(&self, id: crate::event::CommandId, data: Option<Box<dyn std::any::Any + Send>>) {
        self.view.put_command(id, data);
    }

    /// Query the focused child's cursor request and translate to group-relative coords.
    pub fn cursor(&self) -> Option<crate::cursor::CursorRequest> {
        let child = self.focused_child()?;
        let mut req = child.cursor()?;
        let cb = child.bounds();
        let gb = self.view.bounds();
        req.x = req.x.saturating_add(cb.x).saturating_sub(gb.x);
        req.y = req.y.saturating_add(cb.y).saturating_sub(gb.y);
        Some(req)
    }
}

impl Default for GroupState {
    fn default() -> Self {
        Self::new(ViewOptions {
            focusable: true,
            ..ViewOptions::default()
        })
    }
}

#[cfg(test)]
mod tests;
