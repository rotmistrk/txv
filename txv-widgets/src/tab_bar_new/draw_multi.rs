//! TabBar multi-tab draw logic — spread mode with separators and overflow.

use txv_core::prelude::*;

use super::draw::truncate_title;
use super::draw_multi_ctx::MultiDrawCtx;
use super::draw_multi_seg_ctx::SegCtx;
use super::TabBar;

impl TabBar {
    pub(crate) fn draw_multi(&mut self, w: u16) {
        if self.titles.is_empty() {
            return;
        }
        let order = self.display_order();
        self.adjust_scroll_for_active(&order, w);
        let segments = self.compute_segments(&order, w);
        let visible_start = self.scroll_offset;
        let visible_end = visible_start + segments.len();
        let hidden_left = visible_start;
        let hidden_right = order.len().saturating_sub(visible_end);
        let fill_bg = match self.fill.style.bg() {
            Color::Transparent => Color::Reset,
            other => other,
        };

        let mut x = self.draw_left_overflow_multi(0, hidden_left, fill_bg);
        let ctx = MultiDrawCtx {
            order: &order,
            vis_start: visible_start,
            fill_bg,
        };
        let (nx, prev_bg) = self.draw_segments_loop(x, w, &segments, hidden_left, &ctx);
        x = nx;
        if hidden_right == 0 {
            self.draw_trailing_cap_multi(x, w, prev_bg, fill_bg);
        } else {
            self.draw_right_badge(x, w, hidden_right, visible_end, prev_bg, fill_bg);
        }
    }

    fn adjust_scroll_for_active(&mut self, order: &[usize], w: u16) {
        let Some(active_pos) = order.iter().position(|&i| i == self.active) else {
            return;
        };
        if active_pos < self.scroll_offset {
            self.scroll_offset = active_pos;
        } else {
            let segments = self.compute_segments(order, w);
            if active_pos >= self.scroll_offset + segments.len() {
                self.scroll_offset = active_pos;
            }
        }
    }

    fn draw_left_overflow_multi(&mut self, x: u16, hidden_left: usize, fill_bg: Color) -> u16 {
        if hidden_left == 0 {
            return x;
        }
        let indicator = format!("…{hidden_left}");
        let dim = Style::new(self.palette.dim_fg, fill_bg);
        self.state.buffer_mut().print(x, 0, &indicator, dim);
        x + indicator.chars().count() as u16
    }

    fn draw_segments_loop(
        &mut self,
        mut x: u16,
        w: u16,
        segments: &[(usize, String, usize)],
        hidden_left: usize,
        ctx: &MultiDrawCtx,
    ) -> (u16, Color) {
        let mut prev_bg = ctx.fill_bg;
        let mut prev_active = false;
        for (seg_idx, &(order_idx, ref text, text_len)) in segments.iter().enumerate() {
            let tab_idx = ctx.order[order_idx + ctx.vis_start];
            let display_pos = order_idx + ctx.vis_start;
            let ts = self.tab_style(display_pos, tab_idx);
            let is_active = tab_idx == self.active;
            let seg = SegCtx {
                seg_idx,
                hidden_left,
                is_active,
                prev_active,
                cur_bg: ts.bg,
                prev_bg,
            };
            x = self.emit_separator(x, w, &seg, ctx, order_idx);
            x = self.emit_tab_text(x, w, tab_idx, ts, text, text_len);
            prev_bg = ts.bg;
            prev_active = is_active;
        }
        (x, prev_bg)
    }

    /// Compute tab segments that fit in width, starting from scroll_offset.
    fn compute_segments(&self, order: &[usize], avail_w: u16) -> Vec<(usize, String, usize)> {
        let mut segments = Vec::new();
        let mut used = 0u16;
        let badge_reserve = 5u16;
        for (i, &tab_idx) in order.iter().enumerate().skip(self.scroll_offset) {
            let label = self.format_tab_label(i, tab_idx);
            let len = label.chars().count();
            let needed = len as u16 + 1;
            if used + needed > avail_w.saturating_sub(badge_reserve) && !segments.is_empty() {
                break;
            }
            segments.push((i - self.scroll_offset, label, len));
            used += needed;
        }
        segments
    }

    fn format_tab_label(&self, display_pos: usize, tab_idx: usize) -> String {
        let title = &self.titles[tab_idx];
        let display_title = truncate_title(title, 60);
        let num = self.number_label(display_pos, tab_idx);
        let dirty = if self.dirty.get(tab_idx).copied().unwrap_or(false) {
            " •"
        } else {
            ""
        };
        let badge = self.badge_str(tab_idx);
        match num {
            Some(c) => format!("{c}{display_title}{dirty}{badge}"),
            None => format!(" {display_title}{dirty}{badge} "),
        }
    }

    pub(crate) fn badge_str(&self, tab_idx: usize) -> &str {
        match self.badges.get(tab_idx) {
            Some(Some(badge)) => badge.as_str(),
            _ => "",
        }
    }
}
