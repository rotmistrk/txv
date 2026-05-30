//! Group — a View that owns and dispatches to child Views.
//!
//! Three-phase dispatch:
//! 1. Preprocess: children with `options().preprocess` see event first
//! 2. Focused/modal: the modal child (if any) or focused child handles
//! 3. Postprocess: children with `options().postprocess` see event last

mod dispatch;
mod view_fwd;

use std::collections::HashMap;

use crate::view::{View, ViewOptions, ViewState};

/// Common group state — embed in any view that owns children.
pub struct GroupState {
    view: ViewState,
    pub(crate) children: Vec<Box<dyn View>>,
    /// Origin of each child in parent-local coordinates.
    pub(crate) origins: Vec<(u16, u16)>,
    pub(crate) focused: usize,
    /// Named children: name → index.
    named: HashMap<String, usize>,
}

impl GroupState {
    pub fn new(options: ViewOptions) -> Self {
        Self {
            view: ViewState::new(options),
            children: Vec::new(),
            origins: Vec::new(),
            focused: 0,
            named: HashMap::new(),
        }
    }

    pub fn insert(&mut self, child: Box<dyn View>) {
        self.children.push(child);
        self.origins.push((0, 0));
        let idx = self.children.len() - 1;
        self.propagate_sink_to(idx);
        self.view.mark_dirty();
    }

    pub fn insert_at(&mut self, index: usize, child: Box<dyn View>) {
        self.children.insert(index, child);
        self.origins.insert(index, (0, 0));
        self.propagate_sink_to(index);
        self.view.mark_dirty();
    }

    pub fn remove(&mut self, index: usize) -> Box<dyn View> {
        let child = self.children.remove(index);
        self.origins.remove(index);
        if self.focused >= self.children.len() && self.focused > 0 {
            self.focused -= 1;
        }
        // Update named indices that shifted.
        for val in self.named.values_mut() {
            if *val > index {
                *val -= 1;
            }
        }
        self.view.mark_dirty();
        child
    }

    /// Insert a named child. Replaces any existing child with the same name.
    pub fn insert_named(&mut self, name: &str, child: Box<dyn View>) {
        if let Some(&old_idx) = self.named.get(name) {
            self.children[old_idx] = child;
            self.propagate_sink_to(old_idx);
        } else {
            self.children.push(child);
            self.origins.push((0, 0));
            let idx = self.children.len() - 1;
            self.propagate_sink_to(idx);
            self.named.insert(name.to_string(), idx);
        }
        self.view.mark_dirty();
    }

    /// Remove a named child. Returns it if found.
    pub fn remove_named(&mut self, name: &str) -> Option<Box<dyn View>> {
        let idx = self.named.remove(name)?;
        Some(self.remove(idx))
    }

    /// Check if a named child exists.
    pub fn has_named(&self, name: &str) -> bool {
        self.named.contains_key(name)
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

    /// Set origin (position within parent) and size for a child.
    pub fn set_child_bounds(&mut self, index: usize, rect: crate::geometry::Rect) {
        if let Some(origin) = self.origins.get_mut(index) {
            *origin = (rect.x, rect.y);
        }
        if let Some(child) = self.children.get_mut(index) {
            child.set_bounds(crate::geometry::Rect::new(0, 0, rect.w, rect.h));
        }
    }

    /// Set origin of a child in parent-local coordinates.
    pub fn set_child_origin(&mut self, index: usize, x: u16, y: u16) {
        if let Some(origin) = self.origins.get_mut(index) {
            *origin = (x, y);
        }
    }

    /// Get origin of a child in parent-local coordinates.
    pub fn child_origin(&self, index: usize) -> (u16, u16) {
        self.origins.get(index).copied().unwrap_or((0, 0))
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
