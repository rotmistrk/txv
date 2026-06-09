//! Priority-pack layout computation for StatusBar.
//!
//! Algorithm (runs after group.dispatch()):
//! 1. Read each visible child's wanted width (bounds().w)
//! 2. If total > available: hide lowest-priority items until fits
//! 3. If total < available: distribute remaining to stretch items
//! 4. Assign x positions: Left-gravity from left edge, Right-gravity from right edge

use crate::geometry::Rect;

use super::bar::StatusBar;
use super::gravity::Gravity;

struct LayoutItem {
    idx: usize,
    wanted: u16,
    stretch: u16,
    gravity: Gravity,
    priority: u8,
    alloc: u16,
}

impl StatusBar {
    /// Recompute child bounds based on current group bounds and hints.
    pub(super) fn recompute_layout(&mut self) {
        let bounds = self.bounds_rect();
        if bounds.w == 0 {
            return;
        }

        let mut items: Vec<LayoutItem> = self.collect_layout_items();

        // Sort by priority descending (stable by insertion order)
        items.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.idx.cmp(&b.idx)));

        // Hide lowest-priority items that don't fit
        Self::drop_overflow(&mut items, bounds.w);

        // Allocate wanted width, then distribute remaining to stretch
        for item in &mut items {
            item.alloc = item.wanted;
        }
        Self::distribute_stretch(&mut items, bounds.w);

        // Restore insertion order and assign x positions
        items.sort_by_key(|i| i.idx);
        self.assign_positions(&items, bounds);
    }

    fn collect_layout_items(&self) -> Vec<LayoutItem> {
        self.hint_iter()
            .enumerate()
            .filter_map(|(idx, (priority, stretch, gravity, _natural))| {
                let wanted = self.child_wanted_width(idx);
                if wanted == 0 {
                    return None;
                }
                Some(LayoutItem {
                    idx,
                    wanted,
                    stretch,
                    gravity,
                    priority,
                    alloc: 0,
                })
            })
            .collect()
    }

    fn drop_overflow(items: &mut Vec<LayoutItem>, w: u16) {
        let mut total: u16 = items.iter().map(|i| i.wanted).sum();
        while total > w && !items.is_empty() {
            total -= items.last().map(|i| i.wanted).unwrap_or(0);
            items.pop();
        }
    }

    fn distribute_stretch(items: &mut [LayoutItem], w: u16) {
        let used: u16 = items.iter().map(|i| i.alloc).sum();
        let remaining = w.saturating_sub(used);
        if remaining == 0 {
            return;
        }
        let total_stretch: u16 = items.iter().map(|i| i.stretch).sum();
        if total_stretch == 0 {
            return;
        }
        for item in items.iter_mut().filter(|i| i.stretch > 0) {
            let share = (remaining as u32 * item.stretch as u32 / total_stretch as u32) as u16;
            item.alloc += share;
        }
    }

    fn assign_positions(&mut self, items: &[LayoutItem], bounds: Rect) {
        let right_total: u16 = items
            .iter()
            .filter(|i| i.gravity == Gravity::Right)
            .map(|i| i.alloc)
            .sum();
        let mut lx = bounds.x;
        let mut rx = bounds.x + bounds.w.saturating_sub(right_total);

        let mut assigned = vec![false; self.child_count()];

        for item in items {
            match item.gravity {
                Gravity::Left => {
                    if lx + item.alloc <= rx {
                        self.set_child_rect(item.idx, Rect::new(lx, bounds.y, item.alloc, bounds.h));
                        assigned[item.idx] = true;
                        lx += item.alloc;
                    }
                }
                Gravity::Right => {
                    self.set_child_rect(item.idx, Rect::new(rx, bounds.y, item.alloc, bounds.h));
                    assigned[item.idx] = true;
                    rx += item.alloc;
                }
            }
        }

        for (idx, is_assigned) in assigned.iter().enumerate() {
            if !is_assigned {
                self.set_child_rect(idx, Rect::new(0, 0, 0, bounds.h));
            }
        }
    }
}
