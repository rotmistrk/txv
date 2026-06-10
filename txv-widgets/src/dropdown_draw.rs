//! Draw helpers for DropdownMenu.

use txv_core::prelude::*;

use super::dropdown_menu::{DropdownMenu, OpenSide};
use super::dropdown_source::DropdownSource;

const SUBSCRIPTS: [char; 9] = ['₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];

impl<D: DropdownSource> DropdownMenu<D> {
    pub(crate) fn draw_frame(&mut self, w: u16, h: u16, style: Style) {
        let buf = self.state.buffer_mut();
        let draw_top = self.open_side != OpenSide::Top;
        let draw_bottom = self.open_side != OpenSide::Bottom;
        if draw_top {
            buf.put(0, 0, '┌', style);
            for x in 1..w - 1 {
                buf.put(x, 0, '─', style);
            }
            buf.put(w - 1, 0, '┐', style);
        }
        if draw_bottom {
            let y = h - 1;
            buf.put(0, y, '└', style);
            for x in 1..w - 1 {
                buf.put(x, y, '─', style);
            }
            buf.put(w - 1, y, '┘', style);
        }
        let top_row = u16::from(draw_top);
        let bot_row = if draw_bottom {
            h - 1
        } else {
            h
        };
        for y in top_row..bot_row {
            buf.put(0, y, '│', style);
            buf.put(w - 1, y, '│', style);
        }
    }

    pub(crate) fn draw_items(&mut self, w: u16, bg: Style, selected: Style, dim: Style) {
        let top = if self.open_side == OpenSide::Top {
            0
        } else {
            1
        };
        let content_h = self.content_height() as usize;
        let avail_w = w.saturating_sub(4) as usize;
        let hl_fg = palette().style(StyleId::SearchMatch).fg();
        let dim_fg = dim.fg();

        for row in 0..content_h {
            let vis_idx = self.scroll.offset + row;
            if vis_idx >= self.source.visible_len() {
                break;
            }
            let y = top + row as u16;
            let rs = if vis_idx == self.cursor {
                selected
            } else {
                bg
            };
            let ds = Style::new(dim_fg, rs.bg());
            for x in 1..w - 1 {
                self.state.buffer_mut().put(x, y, ' ', rs);
            }
            let x = self.draw_prefix(row, vis_idx, y, rs, ds);
            let label = self.source.label(self.source.visible_index(vis_idx)).to_string();
            self.draw_label(&label, x, y, avail_w, rs, hl_fg);
            self.draw_secondary(vis_idx, y, w, rs, ds);
        }
    }

    fn draw_prefix(&mut self, row: usize, vis_idx: usize, y: u16, _rs: Style, ds: Style) -> u16 {
        use super::dropdown_menu::NumberMode;
        let mut x: u16 = 2;
        if self.number_mode != NumberMode::None {
            let ch = match self.number_mode {
                NumberMode::All => SUBSCRIPTS.get(row).copied().unwrap_or(' '),
                NumberMode::SkipFirst => {
                    if row == 0 {
                        ' '
                    } else {
                        SUBSCRIPTS.get(row - 1).copied().unwrap_or(' ')
                    }
                }
                NumberMode::None => ' ',
            };
            self.state.buffer_mut().put(x, y, ch, ds);
            x += 1;
        }
        // Badge moved to right side — not drawn here
        let _ = vis_idx;
        x
    }

    fn draw_secondary(&mut self, vis_idx: usize, y: u16, w: u16, rs: Style, dim_s: Style) {
        let orig_idx = self.source.visible_index(vis_idx);
        let mut right_x = w - 2;
        // Secondary text (rightmost)
        let sec = self.source.secondary(orig_idx).to_string();
        if !sec.is_empty() {
            let sec_x = right_x.saturating_sub(sec.len() as u16);
            for (i, ch) in sec.chars().enumerate() {
                self.state.buffer_mut().put(sec_x + i as u16, y, ch, dim_s);
            }
            right_x = sec_x.saturating_sub(1);
        }
        // Badge (2-char wide, before secondary)
        if let Some((badge_ch, badge_s)) = self.source.badge(orig_idx) {
            let bs = Style::new(badge_s.fg(), rs.bg());
            let bx = right_x.saturating_sub(1);
            self.state.buffer_mut().put(bx, y, badge_ch, bs);
            // Second position is space (for wide chars)
            self.state.buffer_mut().put(bx + 1, y, ' ', rs);
        }
    }

    fn draw_label(&mut self, label: &str, x: u16, y: u16, avail: usize, base: Style, hl_fg: Color) {
        if self.filter.is_empty() {
            for (i, ch) in label.chars().take(avail).enumerate() {
                self.state.buffer_mut().put(x + i as u16, y, ch, base);
            }
            return;
        }
        let filter_lc: Vec<char> = self
            .filter
            .chars()
            .map(|c| c.to_lowercase().next().unwrap_or(c))
            .collect();
        let mut fi = 0;
        for (i, ch) in label.chars().take(avail).enumerate() {
            let lc = ch.to_lowercase().next().unwrap_or(ch);
            let style = if fi < filter_lc.len() && lc == filter_lc[fi] {
                fi += 1;
                Style::new(hl_fg, base.bg()).with_attrs(base.attrs())
            } else {
                base
            };
            self.state.buffer_mut().put(x + i as u16, y, ch, style);
        }
    }

    pub(crate) fn draw_filter_label(&mut self, w: u16, h: u16, style: Style) {
        use super::dropdown_menu::FilterMode;
        let y = if self.open_side == OpenSide::Bottom {
            0
        } else {
            h - 1
        };
        let indicator = match self.filter_mode {
            FilterMode::None => ' ',
            FilterMode::Prefix => 'ᵖ',
            FilterMode::Substring => 'ˢ',
            FilterMode::Subsequence => 'ᶠ',
        };
        let mut left: u16 = 2;
        if self.filter_mode != FilterMode::None {
            self.state.buffer_mut().put(left, y, indicator, style);
            left += 1;
        }
        let count = format!("{}/{}", self.source.visible_len(), self.source.len());
        let cx = w.saturating_sub(count.len() as u16 + 2);
        for (i, ch) in count.chars().enumerate() {
            self.state.buffer_mut().put(cx + i as u16, y, ch, style);
        }
        if self.filter_enabled && !self.filter.is_empty() {
            let label = format!(" {} ", self.filter);
            for (i, ch) in label.chars().enumerate().take(cx.saturating_sub(left + 1) as usize) {
                self.state.buffer_mut().put(left + i as u16, y, ch, style);
            }
        }
    }
}
