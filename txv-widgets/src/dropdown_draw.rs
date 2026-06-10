//! Draw helpers for DropdownMenu.

use txv_core::prelude::*;

use super::dropdown_menu::{DropdownMenu, OpenSide};
use super::dropdown_source::DropdownSource;

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
        let top_row = if draw_top {
            1
        } else {
            0
        };
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
        let avail_w = w.saturating_sub(3) as usize;
        for row in 0..content_h {
            let vis_idx = self.scroll.offset + row;
            if vis_idx >= self.source.visible_len() {
                break;
            }
            let y = top + row as u16;
            let style = if vis_idx == self.cursor {
                selected
            } else {
                bg
            };
            for x in 1..w - 1 {
                self.state.buffer_mut().put(x, y, ' ', style);
            }
            let mut x: u16 = 2;
            if self.numbers_enabled && row < 9 {
                let ch = char::from_digit((row + 1) as u32, 10).unwrap_or(' ');
                self.state.buffer_mut().put(x, y, ch, dim);
                x += 1;
            }
            let orig_idx = self.source.visible_index(vis_idx);
            if let Some((badge_ch, badge_style)) = self.source.badge(orig_idx) {
                self.state.buffer_mut().put(x, y, badge_ch, badge_style);
                x += 1;
            }
            let label = self.source.label(orig_idx);
            for ch in label.chars().take(avail_w) {
                self.state.buffer_mut().put(x, y, ch, style);
                x += 1;
            }
            let sec = self.source.secondary(orig_idx);
            if !sec.is_empty() {
                let sec_x = (w - 2).saturating_sub(sec.len() as u16);
                if sec_x > x {
                    for (i, ch) in sec.chars().enumerate() {
                        self.state.buffer_mut().put(sec_x + i as u16, y, ch, dim);
                    }
                }
            }
        }
    }

    pub(crate) fn draw_filter_label(&mut self, w: u16, h: u16, style: Style) {
        if !self.filter_enabled || self.filter.is_empty() {
            let count = format!("{}/{}", self.source.visible_len(), self.source.len());
            let x = w.saturating_sub(count.len() as u16 + 2);
            let y = if self.open_side == OpenSide::Bottom {
                0
            } else {
                h - 1
            };
            for (i, ch) in count.chars().enumerate() {
                self.state.buffer_mut().put(x + i as u16, y, ch, style);
            }
            return;
        }
        let y = if self.open_side == OpenSide::Bottom {
            0
        } else {
            h - 1
        };
        let label = format!(" {} ", self.filter);
        let x: u16 = 2;
        for (i, ch) in label.chars().enumerate().take((w - 4) as usize) {
            self.state.buffer_mut().put(x + i as u16, y, ch, style);
        }
    }
}
