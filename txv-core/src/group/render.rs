//! GroupState rendering helpers — blit and visibility.
use crate::buffer::Buffer;

use super::GroupState;

impl GroupState {
    /// Set visibility of a child.
    pub fn set_child_visible(&mut self, index: usize, vis: bool) {
        if let Some(v) = self.visible.get_mut(index) {
            if *v != vis {
                *v = vis;
                self.view.mark_dirty();
            }
        }
    }

    /// Check if a child is visible.
    pub fn is_child_visible(&self, index: usize) -> bool {
        self.visible.get(index).copied().unwrap_or(true)
    }

    /// Blit a single child's buffer onto this group's buffer.
    pub fn blit_child(&mut self, idx: usize) {
        let (ox, oy) = self.child_origin(idx);
        let buf_ptr = self.buffer_mut() as *mut Buffer;
        if let Some(child) = self.child(idx) {
            let cb = child.buffer();
            unsafe { (*buf_ptr).blit(cb, ox, oy) };
        }
    }

    /// Blit all children's buffers onto this group's buffer.
    pub fn blit_all_children(&mut self) {
        for i in 0..self.children.len() {
            self.blit_child(i);
        }
    }
}
