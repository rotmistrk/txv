//! SplitPanel — generic split container with runtime direction switch.
//!
//! Holds 1..N child Views in a proportional split arrangement.
//! Direction (horizontal/vertical) is switchable at runtime.
//! Supports focus cycling and proportional resize.
//! Uses GroupState for three-phase dispatch.

use txv_core::prelude::*;

use crate::tiled_workspace::types::SplitDir;

/// Generic split container.
pub struct SplitPanel {
    group: GroupState,
    proportions: Vec<f32>,
    direction: SplitDir,
    /// If true, row 0 is reserved for chrome (divider starts at row 1).
    chrome_row: bool,
}

impl SplitPanel {
    pub fn new(direction: SplitDir) -> Self {
        Self {
            group: GroupState::new(ViewOptions {
                focusable: true,
                ..ViewOptions::default()
            }),
            proportions: Vec::new(),
            direction,
            chrome_row: false,
        }
    }

    /// Add a child with a given proportion.
    pub fn add_child(&mut self, view: Box<dyn View>, proportion: f32) {
        self.group.insert(view);
        self.proportions.push(proportion);
        self.relayout();
    }

    /// Set the proportion of a child by index.
    pub fn set_proportion(&mut self, idx: usize, proportion: f32) {
        if idx < self.proportions.len() {
            self.proportions[idx] = proportion;
            self.normalize_proportions();
            self.relayout();
        }
    }

    /// Remove a child by index. Returns the removed view.
    pub fn remove_child(&mut self, idx: usize) -> Option<Box<dyn View>> {
        if idx >= self.group.child_count() {
            return None;
        }
        let view = self.group.remove(idx);
        self.proportions.remove(idx);
        self.normalize_proportions();
        self.relayout();
        Some(view)
    }

    /// Number of children.
    pub fn child_count(&self) -> usize {
        self.group.child_count()
    }

    /// Focused child index.
    pub fn focused_index(&self) -> usize {
        self.group.focused_index()
    }

    /// Access a child by index.
    pub fn child(&self, idx: usize) -> Option<&dyn View> {
        self.group.child(idx)
    }

    /// Access a child mutably.
    pub fn child_mut(&mut self, idx: usize) -> Option<&mut Box<dyn View>> {
        self.group.child_mut(idx)
    }

    /// Downcast the focused child to a specific type (immutable).
    pub fn focused_child_as<T: 'static>(&self) -> Option<&T> {
        let child = self.group.focused_child()?;
        child.as_any().and_then(|a| a.downcast_ref::<T>())
    }

    /// Downcast the focused child to a specific type (mutable).
    pub fn focused_child_as_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let child = self.group.focused_child_mut()?;
        child.as_any_mut().and_then(|a| a.downcast_mut::<T>())
    }

    /// Set split direction (relayouts immediately).
    pub fn set_direction(&mut self, dir: SplitDir) {
        if self.direction != dir {
            self.direction = dir;
            self.relayout();
        }
    }

    /// Set whether row 0 is reserved for chrome (divider starts at row 1).
    pub fn set_chrome_row(&mut self, enabled: bool) {
        self.chrome_row = enabled;
    }

    /// Current direction.
    pub fn direction(&self) -> SplitDir {
        self.direction
    }

    /// Cycle focus to the next child.
    pub fn cycle_focus(&mut self) {
        self.group.focus_next();
    }

    /// Focus a specific child.
    pub fn set_focused(&mut self, idx: usize) {
        if idx != self.group.focused_index() {
            self.group.switch_focus(idx);
        }
    }

    /// Grow the focused child's proportion.
    pub fn grow_focused(&mut self) {
        self.adjust_size(0.05);
    }

    /// Shrink the focused child's proportion.
    pub fn shrink_focused(&mut self) {
        self.adjust_size(-0.05);
    }

    fn adjust_size(&mut self, delta: f32) {
        if self.group.child_count() < 2 {
            return;
        }
        let focused = self.group.focused_index();
        let neighbor = if focused + 1 < self.group.child_count() {
            focused + 1
        } else {
            focused - 1
        };
        self.proportions[focused] = (self.proportions[focused] + delta).clamp(0.1, 0.9);
        self.proportions[neighbor] = (self.proportions[neighbor] - delta).clamp(0.1, 0.9);
        self.normalize_proportions();
        self.relayout();
    }

    fn normalize_proportions(&mut self) {
        let total: f32 = self.proportions.iter().sum();
        if total > 0.0 {
            for p in &mut self.proportions {
                *p /= total;
            }
        }
    }

    fn relayout(&mut self) {
        let b = self.group.bounds();
        if b.w == 0 || b.h == 0 || self.group.is_empty() {
            return;
        }
        self.normalize_proportions();
        let count = self.group.child_count();
        let dividers = count.saturating_sub(1) as u16;
        let total_size = match self.direction {
            SplitDir::Horizontal => b.w.saturating_sub(dividers),
            SplitDir::Vertical => b.h,
        };
        let mut offset = 0u16;
        for i in 0..count {
            let is_last = i == count - 1;
            let size = if is_last {
                total_size.saturating_sub(offset)
            } else {
                (total_size as f32 * self.proportions[i]).round() as u16
            };
            let rect = match self.direction {
                SplitDir::Horizontal => {
                    let abs_offset = offset + i as u16;
                    Rect::new(b.x + abs_offset, b.y, size, b.h)
                }
                SplitDir::Vertical => Rect::new(b.x, b.y + offset, b.w, size),
            };
            self.group.set_child_bounds(i, rect);
            offset += size;
        }
        self.group.mark_dirty();
    }
}

mod view_impl;

#[cfg(test)]
mod tests;
