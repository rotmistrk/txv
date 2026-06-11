//! TabBar draw logic — single-tab mode, powerline caps, badge rendering.

use txv_core::prelude::*;

use super::{TabBar, TabBarMode};

/// Truncate a title to fit within `max_chars`, appending `…` if needed.
/// For paths, collapses leading segments: `…/last/segments`.
pub(crate) fn truncate_title(title: &str, max_chars: usize) -> String {
    let char_count = title.chars().count();
    if char_count <= max_chars {
        return title.to_string();
    }
    if max_chars <= 1 {
        return "…".to_string();
    }
    if title.contains('/') {
        let parts: Vec<&str> = title.split('/').collect();
        for skip in 1..parts.len() {
            let candidate = format!("…/{}", parts[skip..].join("/"));
            if candidate.chars().count() <= max_chars {
                return candidate;
            }
        }
    }
    let mut s: String = title.chars().take(max_chars - 1).collect();
    s.push('…');
    s
}

struct SingleCtx<'a> {
    tab_left: &'a str,
    tab_right: &'a str,
    tl_len: u16,
    tr_len: u16,
    fill_bg: Color,
}

impl TabBar {
    pub(crate) fn draw_bar(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        for col in 0..w {
            self.state.buffer_mut().put(col, 0, self.fill.ch, self.fill.style);
        }
        match self.mode {
            TabBarMode::Single => self.draw_single(w),
            TabBarMode::Static | TabBarMode::Lru => self.draw_multi(w),
        }
    }

    fn draw_single(&mut self, w: u16) {
        if self.titles.is_empty() {
            return;
        }
        let g = glyphs();
        let ctx = SingleCtx {
            tab_left: g.chrome().tab_left(),
            tab_right: g.chrome().tab_right(),
            tl_len: g.chrome().tab_left().chars().count() as u16,
            tr_len: g.chrome().tab_right().chars().count() as u16,
            fill_bg: match self.fill.style.bg() {
                Color::Transparent => Color::Reset,
                other => other,
            },
        };
        let ts = self.tab_style(0, self.active);
        let tab_label = self.build_single_label();
        let label_len = tab_label.chars().count() as u16;
        let count_badge = self.build_count_badge(&g);
        let badge_len = count_badge.chars().count() as u16;
        let style = Style::new(ts.fg, ts.bg);

        let x = self.draw_tab_body(w, &ctx, &tab_label, label_len, ts, style);
        self.draw_count_badge_part(x, w, &ctx, &count_badge, badge_len);
    }

    fn build_single_label(&self) -> String {
        let title = truncate_title(&self.titles[self.active], 60);
        let dirty = if self.dirty.get(self.active).copied().unwrap_or(false) {
            " •"
        } else {
            ""
        };
        let badge_text = self.badge_str(self.active);
        format!(" {title}{dirty}{badge_text} ")
    }

    fn build_count_badge(&self, g: &txv_core::glyphs::GlyphSet) -> String {
        let count = self.titles.len();
        if count > 1 {
            format!("{}{count}", g.chrome().dropdown_arrow())
        } else {
            String::new()
        }
    }

    fn draw_tab_body(
        &mut self,
        w: u16,
        ctx: &SingleCtx,
        label: &str,
        label_len: u16,
        ts: super::tab_style::TabStyle,
        style: Style,
    ) -> u16 {
        let badge_len = display_width(self.badge_str(self.active), 8);
        let mut x = 0u16;
        let cap = Style::new(ts.bg, ctx.fill_bg);
        if ctx.tl_len <= w {
            self.state.buffer_mut().print(x, 0, ctx.tab_left, cap);
            x += ctx.tl_len;
        }
        let tab_end = label_len.min(w.saturating_sub(x + badge_len + ctx.tr_len + 2));
        if x + tab_end <= w {
            self.state.buffer_mut().print(x, 0, label, style);
            self.draw_badge_on_tab(x, tab_end, ts.bg);
            x += tab_end;
        }
        let end = Style::new(ts.bg, ctx.fill_bg);
        if x + ctx.tr_len <= w {
            self.state.buffer_mut().print(x, 0, ctx.tab_right, end);
            x += ctx.tr_len;
        }
        x
    }

    fn draw_badge_on_tab(&mut self, x: u16, tab_end: u16, tab_bg: Color) {
        let badge_text = self.badge_str(self.active).to_string();
        if badge_text.is_empty() {
            return;
        }
        let Some(Some(bs)) = self.badge_styles.get(self.active) else {
            return;
        };
        let bt_len = badge_text.chars().count() as u16;
        let badge_x = x + tab_end - bt_len - 1;
        let badge_style = Style::new(bs.fg(), tab_bg);
        self.state.buffer_mut().print(badge_x, 0, &badge_text, badge_style);
    }

    fn draw_count_badge_part(&mut self, mut x: u16, w: u16, ctx: &SingleCtx, badge: &str, badge_len: u16) {
        if badge.is_empty() {
            return;
        }
        let badge_bg = if self.focused {
            self.palette.badge_focused_bg
        } else {
            self.palette.inactive[0].bg
        };
        let badge_fg = self.palette.badge_fg;
        if x + badge_len + ctx.tr_len <= w {
            let cap = Style::new(badge_bg, ctx.fill_bg);
            self.state.buffer_mut().print(x, 0, ctx.tab_left, cap);
            x += ctx.tl_len;
            let bs = Style::new(badge_fg, badge_bg);
            self.state.buffer_mut().print(x, 0, badge, bs);
            x += badge_len;
            let end = Style::new(badge_bg, ctx.fill_bg);
            self.state.buffer_mut().print(x, 0, ctx.tab_right, end);
        } else if x + badge_len <= w {
            let dim = Style::new(self.palette.dim_fg, ctx.fill_bg);
            self.state.buffer_mut().print(x, 0, badge, dim);
        }
    }
}
