//! SplitPanel — generic split container with runtime direction switch.
//!
//! Holds 1..N child Views in a proportional split arrangement.
//! Direction (horizontal/vertical) is switchable at runtime.
//! Supports focus cycling and proportional resize.

use txv_core::prelude::*;

use crate::tiled_workspace::types::SplitDir;

/// Generic split container.
pub struct SplitPanel {
    state: ViewState,
    children: Vec<Box<dyn View>>,
    proportions: Vec<f32>,
    direction: SplitDir,
    focused: usize,
}

impl SplitPanel {
    pub fn new(direction: SplitDir) -> Self {
        Self {
            state: ViewState::new(ViewOptions {
                focusable: true,
                ..ViewOptions::default()
            }),
            children: Vec::new(),
            proportions: Vec::new(),
            direction,
            focused: 0,
        }
    }

    /// Add a child with a given proportion.
    pub fn add_child(&mut self, view: Box<dyn View>, proportion: f32) {
        self.children.push(view);
        self.proportions.push(proportion);
        self.normalize_proportions();
        self.relayout();
    }

    /// Remove a child by index. Returns the removed view.
    pub fn remove_child(&mut self, idx: usize) -> Option<Box<dyn View>> {
        if idx >= self.children.len() {
            return None;
        }
        let view = self.children.remove(idx);
        self.proportions.remove(idx);
        if self.focused >= self.children.len() && self.focused > 0 {
            self.focused -= 1;
        }
        self.normalize_proportions();
        self.relayout();
        Some(view)
    }

    /// Number of children.
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Focused child index.
    pub fn focused_index(&self) -> usize {
        self.focused
    }

    /// Access a child by index.
    pub fn child(&self, idx: usize) -> Option<&dyn View> {
        self.children.get(idx).map(|v| &**v)
    }

    /// Access a child mutably.
    pub fn child_mut(&mut self, idx: usize) -> Option<&mut Box<dyn View>> {
        self.children.get_mut(idx)
    }

    /// Set split direction (relayouts immediately).
    pub fn set_direction(&mut self, dir: SplitDir) {
        if self.direction != dir {
            self.direction = dir;
            self.relayout();
        }
    }

    /// Current direction.
    pub fn direction(&self) -> SplitDir {
        self.direction
    }

    /// Cycle focus to the next child.
    pub fn cycle_focus(&mut self) {
        if self.children.len() > 1 {
            self.focused = (self.focused + 1) % self.children.len();
            self.state.mark_dirty();
        }
    }

    /// Focus a specific child.
    pub fn set_focused(&mut self, idx: usize) {
        if idx < self.children.len() {
            self.focused = idx;
            self.state.mark_dirty();
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
        if self.children.len() < 2 {
            return;
        }
        let neighbor = if self.focused + 1 < self.children.len() {
            self.focused + 1
        } else {
            self.focused - 1
        };
        self.proportions[self.focused] = (self.proportions[self.focused] + delta).clamp(0.1, 0.9);
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
        let b = self.state.bounds();
        if b.w == 0 || b.h == 0 || self.children.is_empty() {
            return;
        }
        let total_size = match self.direction {
            SplitDir::Horizontal => b.w,
            SplitDir::Vertical => b.h,
        };
        let count = self.children.len();
        let mut offset = 0u16;
        for (i, child) in self.children.iter_mut().enumerate() {
            let is_last = i == count - 1;
            let size = if is_last {
                total_size.saturating_sub(offset)
            } else {
                (total_size as f32 * self.proportions[i]) as u16
            };
            let rect = match self.direction {
                SplitDir::Horizontal => Rect::new(b.x + offset, b.y, size, b.h),
                SplitDir::Vertical => Rect::new(b.x, b.y + offset, b.w, size),
            };
            child.set_bounds(rect);
            offset += size;
        }
        self.state.mark_dirty();
    }
}

impl View for SplitPanel {
    delegate_view_state!(state, override { set_bounds, draw, handle });

    fn set_bounds(&mut self, r: Rect) {
        self.state.set_bounds(r);
        self.relayout();
    }

    fn draw(&mut self) {
        let b = self.state.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        self.state.buffer_mut().fill(' ', Style::default());
        let buf_ptr = self.state.buffer_mut() as *mut Buffer;
        for child in &mut self.children {
            child.draw();
            let cb = child.bounds();
            if cb.w > 0 && cb.h > 0 {
                let dx = cb.x.saturating_sub(b.x);
                let dy = cb.y.saturating_sub(b.y);
                unsafe { (*buf_ptr).blit(child.buffer(), dx, dy) };
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if matches!(event, Event::Tick) {
            for child in &mut self.children {
                child.handle(event);
            }
            return HandleResult::Ignored;
        }
        if let Some(child) = self.children.get_mut(self.focused) {
            return child.handle(event);
        }
        HandleResult::Ignored
    }
}

#[cfg(test)]
mod tests;
