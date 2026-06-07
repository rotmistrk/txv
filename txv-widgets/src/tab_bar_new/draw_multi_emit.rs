//! TabBar multi-tab separator and tab-text emitters.

use txv_core::prelude::*;

use super::draw_multi_ctx::MultiDrawCtx;
use super::draw_multi_seg_ctx::SegCtx;
use super::TabBar;

impl TabBar {
    pub(super) fn emit_separator(&mut self, x: u16, w: u16, seg: &SegCtx, ctx: &MultiDrawCtx, order_idx: usize) -> u16 {
        if seg.seg_idx == 0 && seg.hidden_left == 0 {
            return self.emit_first_cap(x, w, seg.cur_bg, ctx.fill_bg);
        }
        if x >= w {
            return x;
        }
        self.emit_inter_sep(x, seg, ctx, order_idx)
    }

    fn emit_first_cap(&mut self, x: u16, w: u16, cur_bg: Color, fill_bg: Color) -> u16 {
        let g = glyphs();
        let tl = g.chrome().tab_left();
        let tl_len = tl.chars().count() as u16;
        if x + tl_len > w {
            return x;
        }
        let cap = Style::new(cur_bg, fill_bg);
        self.state.buffer_mut().print(x, 0, tl, cap);
        x + tl_len
    }

    fn emit_inter_sep(&mut self, x: u16, seg: &SegCtx, ctx: &MultiDrawCtx, order_idx: usize) -> u16 {
        let g = glyphs();
        if seg.prev_active {
            let tr = g.chrome().tab_right();
            let cap = Style::new(seg.prev_bg, seg.cur_bg);
            self.state.buffer_mut().print(x, 0, tr, cap);
            return x + tr.chars().count() as u16;
        }
        if seg.is_active {
            let tl = g.chrome().tab_left();
            let cap = Style::new(seg.cur_bg, seg.prev_bg);
            self.state.buffer_mut().print(x, 0, tl, cap);
            return x + tl.chars().count() as u16;
        }
        self.emit_thin_separator(x, seg.cur_bg, ctx, order_idx)
    }

    fn emit_thin_separator(&mut self, x: u16, cur_bg: Color, ctx: &MultiDrawCtx, order_idx: usize) -> u16 {
        let g = glyphs();
        let active_pos = ctx.order.iter().position(|&i| i == self.active).unwrap_or(0);
        let cur_pos = order_idx + ctx.vis_start;
        let sep = if cur_pos <= active_pos {
            g.chrome().tab_separator_left()
        } else {
            g.chrome().tab_separator()
        };
        let sep_style = Style::new(self.palette.separator_fg, cur_bg);
        self.state.buffer_mut().print(x, 0, sep, sep_style);
        x + sep.chars().count() as u16
    }

    pub(super) fn emit_tab_text(
        &mut self,
        x: u16,
        w: u16,
        tab_idx: usize,
        ts: super::tab_style::TabStyle,
        text: &str,
        text_len: usize,
    ) -> u16 {
        if x + text_len as u16 > w {
            return x;
        }
        let style = Style::new(ts.fg, ts.bg);
        self.state.buffer_mut().print(x, 0, text, style);
        self.emit_tab_badge(x, tab_idx, ts, text_len);
        x + text_len as u16
    }

    fn emit_tab_badge(&mut self, x: u16, tab_idx: usize, ts: super::tab_style::TabStyle, text_len: usize) {
        let Some(Some(badge)) = self.badges.get(tab_idx) else {
            return;
        };
        let Some(Some(bs)) = self.badge_styles.get(tab_idx) else {
            return;
        };
        let badge_len = badge.chars().count() as u16;
        let badge_x = x + text_len as u16 - badge_len;
        let badge_style = Style::new(bs.fg(), ts.bg);
        let badge_copy = badge.clone();
        self.state.buffer_mut().print(badge_x, 0, &badge_copy, badge_style);
    }
}
