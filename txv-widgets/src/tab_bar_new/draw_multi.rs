//! TabBar multi-tab draw logic — spread mode with separators and overflow.

use txv_core::prelude::*;

use super::draw::truncate_title;
use super::TabBar;

impl TabBar {
    pub(crate) fn draw_multi(&mut self, w: u16) {
        if self.titles.is_empty() {
            return;
        }
        let g = glyphs();
        let tab_left = g.chrome.tab_left;
        let tab_right = g.chrome.tab_right;
        let tab_left_len = tab_left.chars().count() as u16;
        let tab_right_len = tab_right.chars().count() as u16;

        let order = self.display_order();
        let total = order.len();

        // Ensure active tab is visible by adjusting scroll_offset
        if let Some(active_pos) = order.iter().position(|&i| i == self.active) {
            if active_pos < self.scroll_offset {
                self.scroll_offset = active_pos;
            } else {
                let segments = self.compute_segments(&order, w);
                let visible_end = self.scroll_offset + segments.len();
                if active_pos >= visible_end {
                    self.scroll_offset = active_pos;
                }
            }
        }

        // Compute visible range with scroll
        let segments = self.compute_segments(&order, w);
        let visible_start = self.scroll_offset;
        let visible_end = visible_start + segments.len();
        let hidden_left = visible_start;
        let hidden_right = total.saturating_sub(visible_end);

        let mut x = 0u16;
        let fill_bg = match self.fill.style.bg {
            Color::Transparent => Color::Reset,
            other => other,
        };

        // Left overflow indicator
        if hidden_left > 0 {
            let indicator = format!("…{hidden_left}");
            let dim = Style {
                fg: self.palette.dim_fg,
                bg: fill_bg,
                ..Style::default()
            };
            self.state.buffer_mut().print(x, 0, &indicator, dim);
            x += indicator.chars().count() as u16;
        }

        // Draw visible tabs
        let mut prev_bg = fill_bg;
        let mut prev_active = false;
        for (seg_idx, &(order_idx, ref text, text_len)) in segments.iter().enumerate() {
            let tab_idx = order[order_idx + visible_start];
            let display_pos = order_idx + visible_start;
            let ts = self.tab_style(display_pos, tab_idx);
            let cur_bg = ts.bg;
            let is_active = tab_idx == self.active;

            // Separator / cap
            if seg_idx == 0 && hidden_left == 0 {
                if x + tab_left_len <= w {
                    let cap = Style {
                        fg: cur_bg,
                        bg: fill_bg,
                        ..Style::default()
                    };
                    self.state.buffer_mut().print(x, 0, tab_left, cap);
                    x += tab_left_len;
                }
            } else if (seg_idx > 0 || hidden_left > 0) && x < w {
                if prev_active {
                    let cap = Style {
                        fg: prev_bg,
                        bg: cur_bg,
                        ..Style::default()
                    };
                    self.state.buffer_mut().print(x, 0, tab_right, cap);
                    x += tab_right_len;
                } else if is_active {
                    let cap = Style {
                        fg: cur_bg,
                        bg: prev_bg,
                        ..Style::default()
                    };
                    self.state.buffer_mut().print(x, 0, tab_left, cap);
                    x += tab_left_len;
                } else {
                    // Between inactive tabs: thin separator
                    let active_pos = order.iter().position(|&i| i == self.active).unwrap_or(0);
                    let cur_pos = order_idx + visible_start;
                    let sep = if cur_pos <= active_pos {
                        g.chrome.tab_separator_left
                    } else {
                        g.chrome.tab_separator
                    };
                    let sep_len = sep.chars().count() as u16;
                    let sep_style = Style {
                        fg: self.palette.separator_fg,
                        bg: cur_bg,
                        ..Style::default()
                    };
                    self.state.buffer_mut().print(x, 0, sep, sep_style);
                    x += sep_len;
                }
            }

            // Tab content
            let style = Style {
                fg: ts.fg,
                bg: ts.bg,
                ..Style::default()
            };
            if x + text_len as u16 <= w {
                self.state.buffer_mut().print(x, 0, text, style);
                // Overlay badge with custom style if set
                if let Some(Some(badge)) = self.badges.get(tab_idx) {
                    if let Some(Some(bs)) = self.badge_styles.get(tab_idx) {
                        let badge_len = badge.chars().count() as u16;
                        let badge_x = x + text_len as u16 - badge_len;
                        let badge_style = Style {
                            fg: bs.fg,
                            bg: ts.bg,
                            ..Style::default()
                        };
                        let badge_copy = badge.clone();
                        self.state.buffer_mut().print(badge_x, 0, &badge_copy, badge_style);
                    }
                }
                x += text_len as u16;
            }
            prev_bg = cur_bg;
            prev_active = is_active;
        }

        // Trailing right cap (only if no overflow badge follows)
        if hidden_right == 0 && x + tab_right_len <= w {
            let arrow_style = Style {
                fg: prev_bg,
                bg: fill_bg,
                ..Style::default()
            };
            self.state.buffer_mut().print(x, 0, tab_right, arrow_style);
            x += tab_right_len;
        }

        // Right overflow badge — rendered as "next tab"
        if hidden_right > 0 && x < w {
            let badge = format!("{}…{hidden_right}", glyphs().chrome.dropdown_arrow);
            let badge_len = badge.chars().count() as u16;
            let next_pos = visible_end.min(9);
            let badge_bg = self.palette.inactive[next_pos].bg;
            let badge_fg = self.palette.badge_fg;

            if x + tab_right_len + badge_len + tab_right_len <= w {
                // Full badge: left cap + badge text + right cap
                let cap = Style {
                    fg: prev_bg,
                    bg: badge_bg,
                    ..Style::default()
                };
                self.state.buffer_mut().print(x, 0, tab_right, cap);
                x += tab_right_len;
                let bs = Style {
                    fg: badge_fg,
                    bg: badge_bg,
                    ..Style::default()
                };
                self.state.buffer_mut().print(x, 0, &badge, bs);
                x += badge_len;
                let end = Style {
                    fg: badge_bg,
                    bg: fill_bg,
                    ..Style::default()
                };
                self.state.buffer_mut().print(x, 0, tab_right, end);
            } else if x + tab_right_len + badge_len <= w {
                // Tight: left cap + badge text (no trailing cap)
                let cap = Style {
                    fg: prev_bg,
                    bg: badge_bg,
                    ..Style::default()
                };
                self.state.buffer_mut().print(x, 0, tab_right, cap);
                x += tab_right_len;
                let bs = Style {
                    fg: badge_fg,
                    bg: badge_bg,
                    ..Style::default()
                };
                self.state.buffer_mut().print(x, 0, &badge, bs);
            } else {
                // Very tight: just badge text with proper background
                let bs = Style {
                    fg: badge_fg,
                    bg: badge_bg,
                    ..Style::default()
                };
                let avail = (w - x) as usize;
                let truncated: String = badge.chars().take(avail).collect();
                self.state.buffer_mut().print(x, 0, &truncated, bs);
            }
        }
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

    /// Format a tab label with optional number prefix, dirty indicator, and badge.
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

    /// Get badge string for a tab (from badges vec).
    pub(crate) fn badge_str(&self, tab_idx: usize) -> &str {
        match self.badges.get(tab_idx) {
            Some(Some(badge)) => badge.as_str(),
            _ => "",
        }
    }
}
