//! SplitPane — two child views with a resizable divider.
//! Uses GroupState: children[0] = first, children[1] = second.

use txv_core::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal, // left | right
    Vertical,   // top / bottom
}

pub struct SplitPane {
    group: GroupState,
    pub direction: SplitDirection,
    pub ratio: f32, // 0.0..=1.0, position of divider
}

impl SplitPane {
    pub fn new(direction: SplitDirection, first: Box<dyn View>, second: Box<dyn View>) -> Self {
        let mut group = GroupState::new(ViewOptions {
            focusable: true,
            ..ViewOptions::default()
        });
        group.insert(first);
        group.insert(second);
        Self {
            group,
            direction,
            ratio: 0.5,
        }
    }

    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio.clamp(0.1, 0.9);
        self.apply_layout();
    }

    pub fn resize(&mut self, delta: i16) {
        let total = match self.direction {
            SplitDirection::Horizontal => self.group.bounds().w,
            SplitDirection::Vertical => self.group.bounds().h,
        } as f32;
        if total > 0.0 {
            self.ratio = (self.ratio + delta as f32 / total).clamp(0.1, 0.9);
            self.apply_layout();
        }
    }

    pub fn focused_index(&self) -> usize {
        self.group.focused_index()
    }

    pub fn child_mut(&mut self, idx: usize) -> Option<&mut Box<dyn View>> {
        self.group.child_mut(idx)
    }

    pub fn child(&self, idx: usize) -> Option<&dyn View> {
        self.group.child(idx)
    }

    pub fn child_count(&self) -> usize {
        self.group.child_count()
    }

    pub fn focus_next(&mut self) {
        self.group.focus_next();
    }

    pub fn focus_prev(&mut self) {
        self.group.focus_prev();
    }

    /// Remove a child by index and return it.
    pub fn remove_child(&mut self, idx: usize) -> Box<dyn View> {
        self.group.remove(idx)
    }

    /// Take a child by index, removing it from the split.
    pub fn take_child(mut self, idx: usize) -> Box<dyn View> {
        self.group.remove(idx)
    }

    fn apply_layout(&mut self) {
        let b = self.group.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        let (r1, r2) = match self.direction {
            SplitDirection::Horizontal => {
                let split = (b.w as f32 * self.ratio) as u16;
                (
                    Rect::new(b.x, b.y, split, b.h),
                    Rect::new(b.x + split + 1, b.y, b.w.saturating_sub(split + 1), b.h),
                )
            }
            SplitDirection::Vertical => {
                let split = (b.h as f32 * self.ratio) as u16;
                (
                    Rect::new(b.x, b.y, b.w, split),
                    Rect::new(b.x, b.y + split + 1, b.w, b.h.saturating_sub(split + 1)),
                )
            }
        };
        self.group.set_child_bounds(0, r1);
        self.group.set_child_bounds(1, r2);
    }
}

impl View for SplitPane {
    delegate_group_state!(group, override { set_bounds, draw, handle });

    fn set_bounds(&mut self, r: Rect) {
        self.group.set_bounds(r);
        self.group.mark_dirty();
        self.apply_layout();
    }

    fn draw(&mut self) {
        let w = self.group.buffer_mut().width();
        let h = self.group.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        self.group.buffer_mut().fill(' ', Style::default());
        let my_bounds = self.group.bounds();

        // Draw and blit children
        for child in self.group.children_iter_mut() {
            child.draw();
        }
        // Blit children into own buffer.
        // Safety: we borrow children (immutable) and view.buf (mutable) which are disjoint fields.
        let buf_ptr = self.group.buffer_mut() as *mut Buffer;
        for i in 0..self.group.child_count() {
            if let Some(child) = self.group.child(i) {
                let cb = child.bounds();
                let dx = cb.x.saturating_sub(my_bounds.x);
                let dy = cb.y.saturating_sub(my_bounds.y);
                unsafe { (*buf_ptr).blit(child.buffer(), dx, dy) };
            }
        }

        // Draw divider
        let dim = palette().style(StyleId::Dim);
        let g = glyphs();
        match self.direction {
            SplitDirection::Horizontal => {
                let x = (w as f32 * self.ratio) as u16;
                self.group.buffer_mut().vline(x, 0, h, g.ui.separator_v, dim);
            }
            SplitDirection::Vertical => {
                let y = (h as f32 * self.ratio) as u16;
                self.group.buffer_mut().hline(0, y, w, g.ui.separator_h, dim);
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        self.group.dispatch(event)
    }
}
