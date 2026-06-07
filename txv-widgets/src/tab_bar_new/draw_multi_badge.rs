//! TabBar multi-tab badge and trailing cap emitters.

use txv_core::prelude::*;

use super::badge_render_ctx::BadgeRenderCtx;
use super::TabBar;

impl TabBar {
    pub(super) fn draw_trailing_cap_multi(&mut self, x: u16, w: u16, prev_bg: Color, fill_bg: Color) -> u16 {
        let g = glyphs();
        let tr = g.chrome().tab_right();
        let tr_len = tr.chars().count() as u16;
        if x + tr_len > w {
            return x;
        }
        let s = Style::new(prev_bg, fill_bg);
        self.state.buffer_mut().print(x, 0, tr, s);
        x + tr_len
    }

    pub(super) fn draw_right_badge(
        &mut self,
        x: u16,
        w: u16,
        hidden_right: usize,
        visible_end: usize,
        prev_bg: Color,
        fill_bg: Color,
    ) {
        if x >= w {
            return;
        }
        let g = glyphs();
        let tr = g.chrome().tab_right();
        let tr_len = tr.chars().count() as u16;
        let badge = format!("{}…{hidden_right}", g.chrome().dropdown_arrow());
        let badge_len = badge.chars().count() as u16;
        let next_pos = visible_end.min(9);
        let badge_bg = self.palette.inactive[next_pos].bg;
        let badge_fg = self.palette.badge_fg;
        let bctx = BadgeRenderCtx {
            tr,
            tr_len,
            badge: &badge,
            badge_len,
            prev_bg,
            badge_bg,
            badge_fg,
        };

        if x + tr_len + badge_len + tr_len <= w {
            self.emit_full_right_badge(x, fill_bg, &bctx);
        } else {
            self.emit_compact_badge(x, w, &bctx);
        }
    }

    fn emit_compact_badge(&mut self, mut x: u16, w: u16, bctx: &BadgeRenderCtx) {
        if x + bctx.tr_len + bctx.badge_len <= w {
            let cap = Style::new(bctx.prev_bg, bctx.badge_bg);
            self.state.buffer_mut().print(x, 0, bctx.tr, cap);
            x += bctx.tr_len;
            let bs = Style::new(bctx.badge_fg, bctx.badge_bg);
            self.state.buffer_mut().print(x, 0, bctx.badge, bs);
        } else {
            let bs = Style::new(bctx.badge_fg, bctx.badge_bg);
            let avail = (w - x) as usize;
            let truncated: String = bctx.badge.chars().take(avail).collect();
            self.state.buffer_mut().print(x, 0, &truncated, bs);
        }
    }

    fn emit_full_right_badge(&mut self, mut x: u16, fill_bg: Color, bctx: &BadgeRenderCtx) {
        let cap = Style::new(bctx.prev_bg, bctx.badge_bg);
        self.state.buffer_mut().print(x, 0, bctx.tr, cap);
        x += bctx.tr_len;
        let bs = Style::new(bctx.badge_fg, bctx.badge_bg);
        self.state.buffer_mut().print(x, 0, bctx.badge, bs);
        x += bctx.badge_len;
        let end = Style::new(bctx.badge_bg, fill_bg);
        self.state.buffer_mut().print(x, 0, bctx.tr, end);
    }
}
