//! Layout computation for TiledWorkspace.

use txv_core::geometry::Rect;

use super::types::{LayoutMode, PanelPosition, SplitDir, SplitNode};
use super::TiledWorkspace;
use crate::split_panel::SplitPanel;

impl TiledWorkspace {
    /// Recompute and apply layout based on current bounds.
    pub fn recompute_layout(&mut self) {
        let bounds = self.group.bounds();
        if bounds.w() == 0 || bounds.h() == 0 {
            return;
        }
        self.update_wide_flag(bounds);

        if let Some(z) = self.zoomed {
            self.apply_zoomed_layout(z, bounds);
            return;
        }

        let layout = if self.is_wide {
            self.wide_layout.clone()
        } else {
            self.narrow_layout.clone()
        };
        self.apply_normal_layout(&layout, bounds);
        self.sync_split_directions();
    }

    fn update_wide_flag(&mut self, bounds: Rect) {
        self.is_wide = match self.layout_mode {
            LayoutMode::Auto => {
                let w = bounds.w();
                if w >= self.wide_threshold {
                    true
                } else if w <= self.narrow_threshold {
                    false
                } else {
                    self.is_wide
                }
            }
            LayoutMode::Wide => true,
            LayoutMode::Narrow => false,
        };
    }

    fn apply_zoomed_layout(&mut self, z: usize, bounds: Rect) {
        for i in 0..self.configs.len() {
            if i == z {
                self.group.set_child_bounds(i, bounds);
            } else {
                self.group.set_child_bounds(i, Rect::default());
            }
        }
    }

    fn apply_normal_layout(&mut self, layout: &SplitNode, bounds: Rect) {
        for i in 0..self.configs.len() {
            self.group.set_child_bounds(i, Rect::default());
        }
        let rects = self.compute_rects(layout, bounds);
        for (id, rect) in rects {
            self.group.set_child_bounds(id, rect);
        }
    }

    fn sync_split_directions(&mut self) {
        for i in 0..self.configs.len() {
            if !self.configs[i].splittable || self.configs[i].position == PanelPosition::Center {
                continue;
            }
            let dir = if self.is_wide {
                SplitDir::Vertical
            } else {
                SplitDir::Horizontal
            };
            if let Some(child) = self.group.child_mut(i) {
                if let Some(sp) = child.as_any_mut().and_then(|a| a.downcast_mut::<SplitPanel>()) {
                    sp.set_direction(dir);
                }
            }
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
            SplitNode::Split { direction, children } => self.compute_split_rects(direction, children, bounds),
        }
    }

    fn compute_split_rects(
        &self,
        direction: &SplitDir,
        children: &[(f32, SplitNode)],
        bounds: Rect,
    ) -> Vec<(usize, Rect)> {
        let visible: Vec<(f32, &SplitNode)> = children
            .iter()
            .filter(|(_, child)| self.has_visible(child))
            .map(|(p, c)| (*p, c))
            .collect();
        if visible.is_empty() {
            return vec![];
        }
        let total: f32 = visible.iter().map(|(p, _)| p).sum();
        let has_gaps = match direction {
            SplitDir::Horizontal => self.h_divider_gaps,
            SplitDir::Vertical => self.v_divider_gaps,
        };
        let dividers = if has_gaps {
            visible.len().saturating_sub(1) as u16
        } else {
            0
        };
        let total_size = match direction {
            SplitDir::Horizontal => bounds.w().saturating_sub(dividers),
            SplitDir::Vertical => bounds.h().saturating_sub(dividers),
        };
        self.distribute_rects(&visible, total, total_size, has_gaps, direction, bounds)
    }

    fn distribute_rects(
        &self,
        visible: &[(f32, &SplitNode)],
        total: f32,
        total_size: u16,
        has_gaps: bool,
        direction: &SplitDir,
        bounds: Rect,
    ) -> Vec<(usize, Rect)> {
        let mut result = Vec::new();
        let mut offset = 0u16;
        for (i, (prop, child)) in visible.iter().enumerate() {
            let normalized = prop / total;
            let is_last = i == visible.len() - 1;
            let size = if is_last {
                total_size.saturating_sub(offset)
            } else {
                (total_size as f32 * normalized) as u16
            };
            let gap = if has_gaps {
                i as u16
            } else {
                0
            };
            let abs_offset = offset + gap;
            let child_bounds = match direction {
                SplitDir::Horizontal => Rect::new(bounds.x() + abs_offset, bounds.y(), size, bounds.h()),
                SplitDir::Vertical => Rect::new(bounds.x(), bounds.y() + abs_offset, bounds.w(), size),
            };
            result.extend(self.compute_rects(child, child_bounds));
            offset += size;
        }
        result
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
        let SplitNode::Split { direction, children } = node else {
            return false;
        };
        let pos = children.iter().position(|(_, c)| c.panel_ids().contains(&target));
        let Some(pos) = pos else {
            return false;
        };
        if *direction == dir && children.len() > 1 {
            Self::apply_proportion_delta(children, pos, delta);
            return true;
        }
        Self::adjust_proportion(&mut children[pos].1, target, dir, delta)
    }

    fn apply_proportion_delta(children: &mut [(f32, SplitNode)], pos: usize, delta: i16) {
        let boundary = if pos + 1 < children.len() {
            pos
        } else {
            pos - 1
        };
        let step = 0.02 * delta.unsigned_abs() as f32;
        if delta > 0 {
            children[boundary].0 = (children[boundary].0 + step).min(0.9);
            children[boundary + 1].0 = (children[boundary + 1].0 - step).max(0.05);
        } else {
            children[boundary].0 = (children[boundary].0 - step).max(0.05);
            children[boundary + 1].0 = (children[boundary + 1].0 + step).min(0.9);
        }
    }
}
