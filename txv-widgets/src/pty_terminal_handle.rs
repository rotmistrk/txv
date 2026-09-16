//! PtyTerminal key/event handling and scroll logic.

use txv_core::prelude::*;

use crate::key_encode::key_to_bytes;
use crate::pty_terminal::PtyTerminal;

impl PtyTerminal {
    pub(crate) fn handle_paste(&mut self, text: &str) -> HandleResult {
        if self.exited {
            return HandleResult::Consumed;
        }
        if let Some(session) = self.session.as_mut() {
            session.write(b"\x1b[200~");
            session.write(text.as_bytes());
            session.write(b"\x1b[201~");
        }
        HandleResult::Consumed
    }

    pub(crate) fn handle_key(&mut self, key: &KeyEvent) -> HandleResult {
        if self.exited {
            return HandleResult::Consumed;
        }
        if key.code() == KeyCode::PageUp {
            return self.scroll_up_page();
        }
        if key.code() == KeyCode::PageDown {
            return self.scroll_down_page();
        }
        // Any other key exits scrollback/pinned mode and goes to PTY
        if self.scroll_offset > 0 || self.pinned_mode {
            self.scroll_offset = 0;
            self.pinned_mode = false;
            self.gap = 0;
            self.state.mark_dirty();
        }
        if let Some(bytes) = key_to_bytes(key) {
            if let Some(session) = self.session.as_mut() {
                session.write(&bytes);
            }
            HandleResult::Consumed
        } else {
            HandleResult::Ignored
        }
    }

    pub(crate) fn scroll_up_page(&mut self) -> HandleResult {
        let h = self.prev_rows as usize;
        let cursor_area = self.cursor_area_lines as usize;

        // If not in pinned mode and we have a cursor area configured, enter pinned mode
        if !self.pinned_mode && cursor_area > 0 && h > cursor_area + 2 {
            self.pinned_mode = true;
            self.gap = 0;
        }

        // Page size: scrollback area height - 1 (for context overlap)
        // In pinned mode: h - cursor_area - 1 (separator) - 1 (overlap)
        // In normal mode: h - 1
        let scrollback_height = if self.pinned_mode && cursor_area > 0 {
            h.saturating_sub(cursor_area + 1) // subtract cursor_area and separator
        } else {
            h
        };
        let page = scrollback_height.saturating_sub(1).max(1);

        let max = self.termbuf.scrollback_len() + self.gap;
        self.scroll_offset = (self.scroll_offset + page).min(max);
        self.state.mark_dirty();
        HandleResult::Consumed
    }

    pub(crate) fn scroll_down_page(&mut self) -> HandleResult {
        let h = self.prev_rows as usize;
        let cursor_area = self.cursor_area_lines as usize;

        // Page size matches scroll_up_page
        let scrollback_height = if self.pinned_mode && cursor_area > 0 {
            h.saturating_sub(cursor_area + 1)
        } else {
            h
        };
        let page = scrollback_height.saturating_sub(1).max(1);

        if self.pinned_mode {
            // Calculate current displayed gap
            let displayed_gap = self.calculate_displayed_gap();

            if self.scroll_offset <= page && displayed_gap == 0 {
                // Exiting pinned mode - back to live view
                self.pinned_mode = false;
                self.scroll_offset = 0;
                self.gap = 0;
            } else {
                // Scrolling down: reduce scroll_offset
                // displayed_gap will decrease as scroll_offset decreases
                self.scroll_offset = self.scroll_offset.saturating_sub(page);
            }
        } else {
            self.scroll_offset = self.scroll_offset.saturating_sub(page);
        }

        self.state.mark_dirty();
        HandleResult::Consumed
    }

    /// Calculate the displayed gap: lines between scrollback bottom and cursor area top.
    pub(crate) fn calculate_displayed_gap(&self) -> usize {
        if !self.pinned_mode {
            return 0;
        }
        let cursor_area = self.cursor_area_lines as usize;
        let grid_rows = self.termbuf.grid_rows() as usize;
        let sb_len = self.termbuf.scrollback_len();
        let total = sb_len + grid_rows;

        // Scrollback view bottom line (exclusive)
        let frozen_total = total.saturating_sub(self.gap);
        let scrollback_bottom = frozen_total.saturating_sub(self.scroll_offset);

        // Cursor area top line
        let cursor_top = total.saturating_sub(cursor_area);

        // Gap is lines between them
        cursor_top.saturating_sub(scrollback_bottom)
    }
}
