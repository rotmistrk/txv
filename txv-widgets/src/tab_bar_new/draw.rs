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
        let g = glyphs();
        let raw_title = &self.titles[self.active];
        let title = truncate_title(raw_title, 60);
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

        let badge_text = self.badge_str(self.active).to_string();
        let label = format!(" {title}{dirty}{badge_text} ");
        let label_len = label.chars().count() as u16;

        let count = self.titles.len();
        let badge = if count > 1 {
            format!("{}{count}", g.chrome.dropdown_arrow)
        } else {
            String::new()
        };
        let badge_len = badge.chars().count() as u16;

        let tab_left = g.chrome.tab_left;
        let tab_right = g.chrome.tab_right;
        let tab_left_len = tab_left.chars().count() as u16;
        let tab_right_len = tab_right.chars().count() as u16;

        let mut x = 0u16;
        let fill_bg = match self.fill.style.bg {
            Color::Transparent => Color::Reset,
            other => other,
        };

        // Left cap
        let cap_style = Style {
            fg: ts.bg,
            bg: fill_bg,
            ..Style::default()
        };
        if x + tab_left_len <= w {
            self.state.buffer_mut().print(x, 0, tab_left, cap_style);
            x += tab_left_len;
        }

        // Tab content
        let tab_end = label_len.min(w.saturating_sub(x + badge_len + tab_right_len + 2));
        if x + tab_end <= w {
            self.state.buffer_mut().print(x, 0, &label, style);
            // Overlay badge with custom style if set
            if !badge_text.is_empty() {
                if let Some(Some(bs)) = self.badge_styles.get(self.active) {
                    let bt_len = badge_text.chars().count() as u16;
                    let badge_x = x + tab_end - bt_len - 1; // -1 for trailing space
                    let badge_style = Style {
                        fg: bs.fg,
                        bg: ts.bg,
                        ..Style::default()
                    };
                    self.state.buffer_mut().print(badge_x, 0, &badge_text, badge_style);
                }
            }
            x += tab_end;
        }

        // Right cap
        if x + tab_right_len <= w {
            let end_style = Style {
                fg: ts.bg,
                bg: fill_bg,
                ..Style::default()
            };
            self.state.buffer_mut().print(x, 0, tab_right, end_style);
            x += tab_right_len;
        }

        // Count badge — rendered as "next tab" with powercap
        if !badge.is_empty() && x + badge_len + tab_right_len <= w {
            let badge_bg = if self.focused {
                self.palette.badge_focused_bg
            } else {
                self.palette.inactive[0].bg
            };
            let badge_fg = self.palette.badge_fg;
            let cap = Style {
                fg: badge_bg,
                bg: fill_bg,
                ..Style::default()
            };
            self.state.buffer_mut().print(x, 0, tab_left, cap);
            x += tab_left_len;
            let badge_style = Style {
                fg: badge_fg,
                bg: badge_bg,
                ..Style::default()
            };
            self.state.buffer_mut().print(x, 0, &badge, badge_style);
            x += badge_len;
            let end = Style {
                fg: badge_bg,
                bg: fill_bg,
                ..Style::default()
            };
            self.state.buffer_mut().print(x, 0, tab_right, end);
        } else if !badge.is_empty() && x + badge_len <= w {
            let dim = Style {
                fg: self.palette.dim_fg,
                bg: fill_bg,
                ..Style::default()
            };
            self.state.buffer_mut().print(x, 0, &badge, dim);
        }
    }
}
