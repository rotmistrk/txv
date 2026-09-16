//! PtyTerminal scrollback rendering and content extraction.

use txv_core::cell::Style;

use crate::pty_terminal::PtyTerminal;

impl PtyTerminal {
    /// Return the last `max_lines` lines from scrollback + visible grid as strings.
    pub fn get_content(&self, max_lines: usize) -> Vec<String> {
        let grid_rows = self.termbuf.grid_rows() as usize;
        let sb_len = self.termbuf.scrollback_len();
        let total = sb_len + grid_rows;
        let take = max_lines.min(total);
        let mut result = Vec::with_capacity(take);
        let start = total.saturating_sub(take);
        for i in start..total {
            let line_str = if i < sb_len {
                if let Some(line) = self.termbuf.scrollback_line(sb_len - 1 - i) {
                    line.iter().map(|c| c.ch()).collect::<String>().trim_end().to_string()
                } else {
                    String::new()
                }
            } else {
                let row = i - sb_len;
                if let Some(line) = self.termbuf.grid_line(row) {
                    line.iter().map(|c| c.ch()).collect::<String>().trim_end().to_string()
                } else {
                    String::new()
                }
            };
            result.push(line_str);
        }
        result
    }

    /// Draw pinned mode: scrollback area + separator + live cursor area.
    pub(crate) fn draw_pinned_mode(&mut self, w: u16, h: u16, cursor_area: u16) {
        let separator_y = h - cursor_area - 1;
        let scrollback_height = separator_y as usize;

        // Draw scrollback content in top area
        self.draw_scrollback_region(0, scrollback_height, w as usize);

        // Draw separator line with gap indicator
        self.draw_gap_separator(separator_y, w);

        // Draw live cursor area at bottom
        self.draw_cursor_area(separator_y + 1, cursor_area, w);
    }

    fn draw_scrollback_region(&mut self, start_y: usize, height: usize, w: usize) {
        let grid_rows = self.termbuf.grid_rows() as usize;
        let sb_len = self.termbuf.scrollback_len();
        let total = sb_len + grid_rows;

        // In pinned mode, scroll_offset is from the frozen position (before gap)
        // The gap contains new lines we haven't scrolled through yet
        let frozen_total = total.saturating_sub(self.gap);
        let bottom_line = frozen_total.saturating_sub(self.scroll_offset);
        let top_line = bottom_line.saturating_sub(height);

        for screen_y in 0..height {
            let line_idx = top_line + screen_y;
            self.draw_scrollback_line(line_idx, start_y + screen_y, w, sb_len);
        }
    }

    fn draw_gap_separator(&mut self, y: u16, w: u16) {
        use txv_core::cell::Attrs;
        use txv_core::palette::{palette, StyleId};

        let base = palette().style(StyleId::StateInfo);
        let style = base.with_attrs(Attrs::default().bold());

        // Calculate actual gap between scrollback bottom and cursor area top
        let displayed_gap = self.calculate_displayed_gap();

        // Build separator: ─────< 45 lines >─────
        let gap_text = format!(" {} lines ", displayed_gap);
        let left_arrow = '\u{e0b2}'; // Powerline left arrow
        let right_arrow = '\u{e0b0}'; // Powerline right arrow
        let label = format!("{left_arrow}{gap_text}{right_arrow}");
        let label_len = label.chars().count();

        let line_char = '─';
        let left_pad = (w as usize).saturating_sub(label_len) / 2;
        let right_pad = (w as usize).saturating_sub(label_len + left_pad);

        let mut x = 0u16;
        for _ in 0..left_pad {
            self.state.buffer_mut().put(x, y, line_char, style);
            x += 1;
        }
        for ch in label.chars() {
            if x < w {
                self.state.buffer_mut().put(x, y, ch, style);
                x += 1;
            }
        }
        for _ in 0..right_pad {
            if x < w {
                self.state.buffer_mut().put(x, y, line_char, style);
                x += 1;
            }
        }
    }

    fn draw_cursor_area(&mut self, start_y: u16, height: u16, w: u16) {
        let grid_rows = self.termbuf.grid_rows();
        // Show the last `height` lines of the live grid
        let start_row = grid_rows.saturating_sub(height) as usize;

        for i in 0..height {
            let row = start_row + i as usize;
            if let Some(line) = self.termbuf.grid_line(row) {
                for (x, tc) in line.iter().enumerate().take(w as usize) {
                    self.state.buffer_mut().put(x as u16, start_y + i, tc.ch(), tc.style());
                }
            } else {
                // Fill empty rows with spaces
                let style = Style::default();
                for x in 0..w {
                    self.state.buffer_mut().put(x, start_y + i, ' ', style);
                }
            }
        }
    }

    pub(crate) fn draw_scrollback_to_buf(&mut self) {
        let h = self.state.buffer_mut().height() as usize;
        let w = self.state.buffer_mut().width() as usize;
        let grid_rows = self.termbuf.grid_rows() as usize;
        let sb_len = self.termbuf.scrollback_len();
        let total = sb_len + grid_rows;
        let bottom_line = total.saturating_sub(self.scroll_offset);
        let top_line = bottom_line.saturating_sub(h);
        for screen_y in 0..h {
            let line_idx = top_line + screen_y;
            self.draw_scrollback_line(line_idx, screen_y, w, sb_len);
        }
    }

    fn draw_scrollback_line(&mut self, line_idx: usize, screen_y: usize, w: usize, sb_len: usize) {
        if line_idx < sb_len {
            if let Some(line) = self.termbuf.scrollback_line(sb_len - 1 - line_idx) {
                for (x, tc) in line.iter().enumerate().take(w) {
                    self.state
                        .buffer_mut()
                        .put(x as u16, screen_y as u16, tc.ch(), tc.style());
                }
            }
            return;
        }
        let row = line_idx - sb_len;
        if let Some(line) = self.termbuf.grid_line(row) {
            for (x, tc) in line.iter().enumerate().take(w) {
                self.state
                    .buffer_mut()
                    .put(x as u16, screen_y as u16, tc.ch(), tc.style());
            }
        } else {
            let style = Style::default();
            for x in 0..w {
                self.state.buffer_mut().put(x as u16, screen_y as u16, ' ', style);
            }
        }
    }
}
