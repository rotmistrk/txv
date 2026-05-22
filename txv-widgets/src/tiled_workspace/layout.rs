//! Layout computation for TiledWorkspace.

use txv_core::geometry::Rect;

use super::types::{SplitDir, SplitNode};
use super::TiledWorkspace;

impl TiledWorkspace {
    /// Recompute and apply layout based on current bounds.
    pub(crate) fn recompute_layout(&mut self) {
        let bounds = self.group.bounds();
        if bounds.w == 0 || bounds.h == 0 {
            return;
        }
        self.is_wide = bounds.w >= self.wide_threshold;

        if let Some(z) = self.zoomed {
            // Zoomed: give full bounds to the zoomed panel, zero others
            for i in 0..self.configs.len() {
                if i == z {
                    self.group.set_child_bounds(i, bounds);
                } else {
                    self.group.set_child_bounds(i, Rect::default());
                }
            }
            return;
        }

        let layout = if self.is_wide {
            self.wide_layout.clone()
        } else {
            self.narrow_layout.clone()
        };

        // Zero all panels first, then assign visible ones
        for i in 0..self.configs.len() {
            self.group.set_child_bounds(i, Rect::default());
        }
        let rects = self.compute_rects(&layout, bounds);
        for (id, rect) in rects {
            self.group.set_child_bounds(id, rect);
        }
    }

    /// Compute panel rects from a split tree, skipping hidden panels.
    fn compute_rects(&self, node: &SplitNode, bounds: Rect) -> Vec<(usize, Rect)> {
        match node {
            SplitNode::Leaf(id) => {
                if self.hidden[*id] {
                    vec![]
                } else {
                    vec![(*id, bounds)]
                }
            }
            SplitNode::Split { direction, children } => {
                // Filter out hidden subtrees and redistribute proportions
                let visible: Vec<(f32, &SplitNode)> = children
                    .iter()
                    .filter(|(_, child)| self.has_visible(child))
                    .map(|(p, c)| (*p, c))
                    .collect();
                if visible.is_empty() {
                    return vec![];
                }
                let total: f32 = visible.iter().map(|(p, _)| p).sum();
                let mut result = Vec::new();
                let mut offset = 0u16;
                let total_size = match direction {
                    SplitDir::Horizontal => bounds.w,
                    SplitDir::Vertical => bounds.h,
                };

                for (i, (prop, child)) in visible.iter().enumerate() {
                    let normalized = prop / total;
                    let is_last = i == visible.len() - 1;
                    let size = if is_last {
                        total_size.saturating_sub(offset)
                    } else {
                        (total_size as f32 * normalized) as u16
                    };
                    let child_bounds = match direction {
                        SplitDir::Horizontal => Rect::new(bounds.x + offset, bounds.y, size, bounds.h),
                        SplitDir::Vertical => Rect::new(bounds.x, bounds.y + offset, bounds.w, size),
                    };
                    result.extend(self.compute_rects(child, child_bounds));
                    offset += size;
                }
                result
            }
        }
    }

    /// Check if a subtree has any visible panels.
    fn has_visible(&self, node: &SplitNode) -> bool {
        match node {
            SplitNode::Leaf(id) => !self.hidden[*id],
            SplitNode::Split { children, .. } => children.iter().any(|(_, c)| self.has_visible(c)),
        }
    }

    /// Resize: adjust proportion between adjacent panels in the active layout.
    pub fn resize_panel(&mut self, direction: SplitDir, delta: i16) {
        let focused = self.group.focused_index();
        let layout = if self.is_wide {
            &mut self.wide_layout
        } else {
            &mut self.narrow_layout
        };
        Self::adjust_proportion(layout, focused, direction, delta);
        self.recompute_layout();
    }

    fn adjust_proportion(node: &mut SplitNode, target: usize, dir: SplitDir, delta: i16) -> bool {
        if let SplitNode::Split { direction, children } = node {
            // Find which child contains the target
            let pos = children.iter().position(|(_, c)| c.panel_ids().contains(&target));
            let Some(pos) = pos else {
                return false;
            };

            if *direction == dir && children.len() > 1 {
                // Adjust boundary between pos and pos+1 (or pos-1)
                let neighbor = if delta > 0 && pos + 1 < children.len() {
                    pos + 1
                } else if delta < 0 && pos > 0 {
                    pos - 1
                } else {
                    // Try recursing into the child
                    return Self::adjust_proportion(&mut children[pos].1, target, dir, delta);
                };
                let step = 0.02 * delta.unsigned_abs() as f32;
                let (lo, hi) = if pos < neighbor {
                    (pos, neighbor)
                } else {
                    (neighbor, pos)
                };
                let grow_idx = if delta > 0 {
                    lo
                } else {
                    hi
                };
                let shrink_idx = if delta > 0 {
                    hi
                } else {
                    lo
                };
                children[grow_idx].0 = (children[grow_idx].0 + step).min(0.9);
                children[shrink_idx].0 = (children[shrink_idx].0 - step).max(0.05);
                return true;
            }
            // Recurse into the child containing target
            Self::adjust_proportion(&mut children[pos].1, target, dir, delta)
        } else {
            false
        }
    }
}
