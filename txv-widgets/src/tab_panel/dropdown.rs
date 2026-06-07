//! Dropdown rendering for TabPanel.

use txv_core::prelude::*;

use super::dd_ctx::DdCtx;
use super::TabPanel;

impl TabPanel {
    /// Draw the dropdown into the panel's buffer (below row 0).
    pub(crate) fn draw_dropdown(&mut self) {
        let entries = self.bar().dropdown_entries();
        if entries.is_empty() {
            return;
        }
        let cursor = self.bar().dropdown_cursor.unwrap_or(0);
        let filter = self.bar().dropdown_filter().to_string();
        let border_style = self.dd_border_style();
        let (badge_styles, extra) = self.dd_badge_info(&entries);
        let (buf_w, buf_h) = (self.group.buffer().width(), self.group.buffer().height());
        let visible = entries.len().min((buf_h.saturating_sub(2)) as usize);
        let (box_w, inner_w) = self.dd_dims(&entries, visible, extra, buf_w);
        let start_y = if filter.is_empty() {
            1u16
        } else {
            2u16
        };
        let ctx = DdCtx {
            box_w,
            inner_w,
            buf_h,
            border_style,
        };

        if !filter.is_empty() {
            self.draw_dd_filter(&filter, box_w, inner_w, buf_w, buf_h, border_style);
        }
        self.draw_dd_rows(&entries, visible, cursor, start_y, &ctx, &badge_styles);
        self.draw_dd_bottom(start_y + visible as u16, box_w, buf_w, buf_h, border_style);
    }

    fn dd_border_style(&self) -> Style {
        let active_bg = self.bar().active_tab_bg();
        Style::new(active_bg, Color::Reset)
    }

    fn dd_badge_info(&self, entries: &[(usize, String, bool)]) -> (Vec<Option<Style>>, usize) {
        let styles: Vec<Option<Style>> = entries
            .iter()
            .take(50)
            .map(|(idx, _, _)| self.bar().badge_styles.get(*idx).cloned().flatten())
            .collect();
        let extra = if styles.iter().any(|s| s.is_some()) {
            2
        } else {
            0
        };
        (styles, extra)
    }

    fn dd_dims(&self, entries: &[(usize, String, bool)], visible: usize, extra: usize, buf_w: u16) -> (u16, u16) {
        let cw = entries
            .iter()
            .take(visible)
            .map(|(_, l, _)| l.chars().count() + 4 + extra)
            .max()
            .unwrap_or(10) as u16;
        let bw = (cw + 2).min(buf_w.saturating_sub(1));
        (bw, bw.saturating_sub(2))
    }

    fn draw_dd_filter(&mut self, filter: &str, box_w: u16, inner_w: u16, buf_w: u16, buf_h: u16, border_style: Style) {
        let y = 1u16;
        if y >= buf_h {
            return;
        }
        let pal = palette();
        let fs = Style::new(pal.style(StyleId::CursorFocused).fg(), pal.style(StyleId::Text).bg());
        let g = glyphs();
        let buf = self.group.buffer_mut();
        if 1 < buf_w {
            buf.put(1, y, g.box_drawing().v(), border_style);
        }
        for col in 0..inner_w {
            buf.put(2 + col, y, ' ', fs);
        }
        let text = format!(" /{} ", filter);
        let trunc: String = text.chars().take(inner_w as usize).collect();
        buf.print(2, y, &trunc, fs);
        if box_w < buf_w {
            buf.put(box_w, y, g.box_drawing().v(), border_style);
        }
    }

    fn draw_dd_rows(
        &mut self,
        entries: &[(usize, String, bool)],
        visible: usize,
        cursor: usize,
        start_y: u16,
        ctx: &DdCtx,
        badge_styles: &[Option<Style>],
    ) {
        let pal = palette();
        let normal = pal.style(StyleId::Text);
        let selected = pal.style(StyleId::CursorFocused);
        for (row, (_, label, numbered)) in entries.iter().take(visible).enumerate() {
            let y = start_y + row as u16;
            if y >= ctx.buf_h {
                break;
            }
            let is_cursor = row == cursor;
            let style = if is_cursor {
                selected
            } else {
                normal
            };
            let dot = if is_cursor {
                '·'
            } else {
                ' '
            };
            let padded = if !numbered {
                format!("{dot}  {label}")
            } else {
                format!("{dot}{label}")
            };
            let bs = badge_styles.get(row).and_then(|s| s.as_ref());
            self.draw_dd_row(y, is_cursor, &padded, style, ctx, bs);
        }
    }

    fn draw_dd_row(&mut self, y: u16, is_cursor: bool, padded: &str, style: Style, ctx: &DdCtx, badge: Option<&Style>) {
        let g = glyphs();
        let buf = self.group.buffer_mut();
        buf.put(1, y, g.box_drawing().v(), ctx.border_style);
        let trunc: String = padded.chars().take((ctx.inner_w as usize).saturating_sub(1)).collect();
        for col in 0..ctx.inner_w {
            buf.put(2 + col, y, ' ', style);
        }
        buf.print(2, y, &trunc, style);
        if is_cursor && ctx.inner_w > 0 {
            buf.put(1 + ctx.inner_w, y, '·', style);
        }
        Self::draw_dd_badge(buf, y, ctx.inner_w, style, badge);
        if ctx.box_w < buf.width() {
            buf.put(ctx.box_w, y, g.box_drawing().v(), ctx.border_style);
        }
    }

    fn draw_dd_badge(buf: &mut Buffer, y: u16, inner_w: u16, style: Style, badge: Option<&Style>) {
        let Some(bs) = badge else {
            return;
        };
        let bstyle = Style::new(bs.fg(), style.bg());
        let bx = 1 + inner_w.saturating_sub(1);
        if bx < buf.width() {
            buf.put(bx, y, '●', bstyle);
        }
    }

    fn draw_dd_bottom(&mut self, y_bot: u16, box_w: u16, buf_w: u16, buf_h: u16, border_style: Style) {
        if y_bot >= buf_h {
            return;
        }
        let g = glyphs();
        let buf = self.group.buffer_mut();
        buf.put(1, y_bot, g.box_drawing().bl_round(), border_style);
        for bx in 1..box_w.saturating_sub(1) {
            let col = 1 + bx;
            if col < buf_w {
                buf.put(col, y_bot, g.box_drawing().h(), border_style);
            }
        }
        if box_w < buf_w {
            buf.put(box_w, y_bot, g.box_drawing().br_round(), border_style);
        }
    }
}
