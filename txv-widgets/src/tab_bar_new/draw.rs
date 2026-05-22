//! TabBar draw logic — powerline separators, palette lookup, overflow.

use txv_core::prelude::*;

use super::{TabBar, TabBarMode};

/// Powerline glyphs.
const PL_SOLID: char = '\u{E0B0}'; //
const PL_THIN: char = '\u{E0B1}'; //

impl TabBar {
    pub(crate) fn draw_bar(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }

        // Fill entire bar with fill style
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
        let title = &self.titles[self.active];
        let ts = self.tab_style(0, self.active);
        let style = Style {
            fg: ts.fg,
            bg: ts.bg,
            ..Style::default()
        };

        let dirty = if self.dirty.get(self.active).copied().unwrap_or(false) {
            " •"
        } else {
            ""
        };
        let label = format!(" {title}{dirty} ");
        let label_len = label.chars().count() as u16;

        // Badge: ▾N
        let count = self.titles.len();
        let badge = if count > 1 {
            format!("▾{count}")
        } else {
            String::new()
        };
        let badge_len = badge.chars().count() as u16;

        let mut x = 0u16;
        // Draw active tab
        let tab_end = label_len.min(w.saturating_sub(badge_len + 2));
        self.state.buffer_mut().print(x, 0, &label[..], style);
        x += tab_end;

        // Powerline arrow after active
        if x < w {
            let arrow_style = Style {
                fg: ts.bg,
                bg: self.fill.style.bg,
                ..Style::default()
            };
            self.state.buffer_mut().put(x, 0, PL_SOLID, arrow_style);
            x += 1;
        }

        // Badge at end
        if !badge.is_empty() && badge_len + x < w {
            let dim = Style {
                fg: self.palette.dim_fg,
                bg: self.fill.style.bg,
                ..Style::default()
            };
            self.state.buffer_mut().print(x, 0, &badge, dim);
        }
    }

    fn draw_multi(&mut self, w: u16) {
        if self.titles.is_empty() {
            return;
        }
        let order = self.display_order();
        let total = order.len();

        // Compute visible range with scroll
        let segments = self.compute_segments(&order, w);
        let visible_start = self.scroll_offset;
        let visible_end = visible_start + segments.len();
        let hidden_left = visible_start;
        let hidden_right = total.saturating_sub(visible_end);

        let mut x = 0u16;

        // Left overflow indicator
        if hidden_left > 0 {
            let indicator = format!("…{hidden_left}");
            let dim = Style {
                fg: self.palette.dim_fg,
                bg: self.fill.style.bg,
                ..Style::default()
            };
            self.state.buffer_mut().print(x, 0, &indicator, dim);
            x += indicator.chars().count() as u16;
        }

        // Draw visible tabs
        let mut prev_bg = self.fill.style.bg;
        for (seg_idx, &(order_idx, ref text, text_len)) in segments.iter().enumerate() {
            let tab_idx = order[order_idx + visible_start];
            let display_pos = order_idx + visible_start;
            let ts = self.tab_style(display_pos, tab_idx);
            let cur_bg = ts.bg;

            // Powerline separator
            if (seg_idx > 0 || hidden_left > 0) && x < w {
                if prev_bg != cur_bg {
                    let sep_style = Style {
                        fg: prev_bg,
                        bg: cur_bg,
                        ..Style::default()
                    };
                    self.state.buffer_mut().put(x, 0, PL_SOLID, sep_style);
                } else {
                    let sep_style = Style {
                        fg: self.palette.dim_fg,
                        bg: cur_bg,
                        ..Style::default()
                    };
                    self.state.buffer_mut().put(x, 0, PL_THIN, sep_style);
                }
                x += 1;
            }

            // Tab content
            let style = Style {
                fg: ts.fg,
                bg: ts.bg,
                ..Style::default()
            };
            if x + text_len as u16 <= w {
                self.state.buffer_mut().print(x, 0, text, style);
                x += text_len as u16;
            }
            prev_bg = cur_bg;
        }

        // Trailing powerline arrow
        if x < w {
            let arrow_style = Style {
                fg: prev_bg,
                bg: self.fill.style.bg,
                ..Style::default()
            };
            self.state.buffer_mut().put(x, 0, PL_SOLID, arrow_style);
            x += 1;
        }

        // Right overflow badge
        if hidden_right > 0 && x < w {
            let badge = format!("▾…{hidden_right}");
            let dim = Style {
                fg: self.palette.dim_fg,
                bg: self.fill.style.bg,
                ..Style::default()
            };
            self.state.buffer_mut().print(x, 0, &badge, dim);
        }
    }

    /// Compute tab segments that fit in width, starting from scroll_offset.
    /// Returns Vec<(order_index, rendered_text, char_count)>.
    fn compute_segments(&self, order: &[usize], avail_w: u16) -> Vec<(usize, String, usize)> {
        let mut segments = Vec::new();
        let mut used = 0u16;
        let badge_reserve = 5u16; // room for ▾…NN

        for (i, &tab_idx) in order.iter().enumerate().skip(self.scroll_offset) {
            let label = self.format_tab_label(i, tab_idx);
            let len = label.chars().count();
            let needed = len as u16 + 1; // +1 for separator

            if used + needed > avail_w.saturating_sub(badge_reserve) && !segments.is_empty() {
                break;
            }
            segments.push((i - self.scroll_offset, label, len));
            used += needed;
        }
        segments
    }

    /// Format a tab label with optional number prefix and dirty indicator.
    fn format_tab_label(&self, display_pos: usize, tab_idx: usize) -> String {
        let title = &self.titles[tab_idx];
        let num = self.number_label(display_pos, tab_idx);
        let dirty = if self.dirty.get(tab_idx).copied().unwrap_or(false) {
            " •"
        } else {
            ""
        };
        match num {
            Some(c) => format!("{c}{title}{dirty}"),
            None => format!(" {title}{dirty} "),
        }
    }
}
