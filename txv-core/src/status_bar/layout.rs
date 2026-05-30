//! Priority-pack layout computation for StatusBar.

use crate::geometry::Rect;

use super::bar::StatusBar;
use super::gravity::Gravity;

struct LayoutItem {
    idx: usize,
    min_w: u16,
    max_w: u16,
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

        let mut items: Vec<LayoutItem> = self.collect_layout_items(bounds);

        // Sort by priority descending, stable by insertion order
        items.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.idx.cmp(&b.idx)));

        // Drop lowest-priority items until total min fits
        Self::drop_overflow(&mut items, bounds.w);

        // Allocate min_width
        for item in &mut items {
            item.alloc = item.min_w;
        }

        // Distribute remaining space to stretch items
        Self::distribute_stretch(&mut items, bounds.w);

        // Restore insertion order and assign positions
        items.sort_by_key(|i| i.idx);
        self.assign_positions(&items, bounds);
    }

    fn drop_overflow(items: &mut Vec<LayoutItem>, w: u16) {
        let mut total: u16 = items.iter().map(|i| i.min_w).sum();
        while total > w && !items.is_empty() {
            total -= items.last().map(|i| i.min_w).unwrap_or(0);
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
            let capped = if item.max_w > 0 {
                share.min(item.max_w.saturating_sub(item.alloc))
            } else {
                share
            };
            item.alloc += capped;
        }
    }

    fn collect_layout_items(&self, _bounds: Rect) -> Vec<LayoutItem> {
        self.hint_iter()
            .enumerate()
            .map(
                |(idx, (priority, min_width, max_width, stretch, gravity, natural_width, last_alloc))| {
                    let min_w = if min_width > 0 {
                        min_width
                    } else {
                        let current = self.child_buffer_width(idx);
                        if stretch > 0 && current == last_alloc {
                            // Child width equals last layout allocation — it's inflated
                            // by stretch. Use natural width as the true minimum.
                            natural_width
                        } else if current > 0 {
                            current
                        } else {
                            natural_width
                        }
                    };
                    LayoutItem {
                        idx,
                        min_w,
                        max_w: max_width,
                        stretch,
                        gravity,
                        priority,
                        alloc: 0,
                    }
                },
            )
            .filter(|item| item.min_w > 0)
            .collect()
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
            self.set_last_alloc(item.idx, item.alloc);
        }

        for (idx, is_assigned) in assigned.iter().enumerate() {
            if !is_assigned {
                self.set_child_rect(idx, Rect::new(0, 0, 0, bounds.h));
                self.set_last_alloc(idx, 0);
            }
        }
    }
}
