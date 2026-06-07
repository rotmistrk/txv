//! Chrome drawing — divider lines between panels.
//!
//! Complexity: O(n) where n = number of panels. Independent of screen size.
//! We compute gap coordinates directly from panel rects, then issue
//! hline/vline/put calls — each O(1) amortized by the buffer.

use txv_core::palette::palette;
use txv_core::prelude::*;

use super::TiledWorkspace;

/// A vertical gap segment: column x, from y_start to y_end (exclusive).
struct VGap {
    x: u16,
    y_start: u16,
    y_end: u16,
}

impl TiledWorkspace {
    /// Draw chrome dividers between panels. O(n) in panel count.
    pub(super) fn draw_chrome(&mut self) {
        let style = palette().style(StyleId::ChromeBar);
        let origin = self.group.bounds();
        let w = self.group.buffer_mut().width();
        let h = self.group.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }

        if self.zoomed.is_some() {
            self.group.buffer_mut().hline(0, 0, w, '─', style);
            return;
        }

        let rects = self.collect_panel_rects(origin);
        let (tier_ys, vgaps) = Self::compute_chrome_coords(&rects);
        let buf = self.group.buffer_mut();

        // Horizontal lines at each tier boundary — one hline call per tier
        for &y in &tier_ys {
            buf.hline(0, y, w, '─', style);
        }

        // Vertical gaps — one vline + connector puts per gap
        for gap in &vgaps {
            // Draw │ between tier lines
            let first_tier = tier_ys[0];
            let body_start = gap.y_start + 1;
            let body_end = gap.y_end.min(h);
            if body_start < body_end {
                buf.vline(gap.x, body_start, body_end - body_start, '│', style);
            }
            // Connector at top tier: ┬
            if gap.y_start == first_tier {
                buf.put(gap.x, gap.y_start, '┬', style);
            }
            // Connector at bottom tier boundary (if gap ends at a tier)
            if gap.y_end < h && tier_ys.contains(&gap.y_end) {
                buf.put(gap.x, gap.y_end, '┴', style);
            }
        }
    }

    fn collect_panel_rects(&self, _origin: Rect) -> Vec<Rect> {
        (0..self.configs.len())
            .filter(|&i| !self.hidden[i])
            .filter_map(|i| {
                let child = self.group.child(i)?;
                let cs = child.bounds();
                if cs.w() == 0 || cs.h() == 0 {
                    return None;
                }
                let (ox, oy) = self.group.child_origin(i);
                Some(Rect::new(ox, oy, cs.w(), cs.h()))
            })
            .collect()
    }

    /// Compute chrome coordinates from panel rects. O(n²) where n = panel count (≤8).
    fn compute_chrome_coords(rects: &[Rect]) -> (Vec<u16>, Vec<VGap>) {
        let mut tier_ys: Vec<u16> = rects.iter().map(|r| r.y()).collect();
        tier_ys.sort_unstable();
        tier_ys.dedup();

        let mut gaps: Vec<VGap> = Vec::new();
        for a in rects {
            let gap_x = a.x() + a.w();
            // Check if another panel starts immediately after the gap
            let has_neighbor = rects.iter().any(|b| b.x() == gap_x + 1 && b.y() == a.y());
            if !has_neighbor {
                continue;
            }
            if gaps.iter().any(|g| g.x == gap_x && g.y_start == a.y()) {
                continue;
            }
            gaps.push(VGap {
                x: gap_x,
                y_start: a.y(),
                y_end: a.y() + a.h(),
            });
        }

        (tier_ys, gaps)
    }
}
