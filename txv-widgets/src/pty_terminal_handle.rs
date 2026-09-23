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
            self.pinned_bottom_line = 0;
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

    /// Get current total lines (scrollback + grid)
    fn total_lines(&self) -> usize {
        self.termbuf.scrollback_len() + self.termbuf.grid_rows() as usize
    }

    /// Calculate scrollback area height (top part of split view)
    fn scrollback_area_height(&self) -> usize {
        let h = self.prev_rows as usize;
        let cursor_area = self.cursor_area_lines as usize;
        if self.pinned_mode && cursor_area > 0 {
            h.saturating_sub(cursor_area + 1) // subtract cursor_area and separator
        } else {
            h
        }
    }

    pub(crate) fn scroll_up_page(&mut self) -> HandleResult {
        let h = self.prev_rows as usize;
        let cursor_area = self.cursor_area_lines as usize;
        let total = self.total_lines();

        // If not in pinned mode and we have a cursor area configured, enter pinned mode
        if !self.pinned_mode && cursor_area > 0 && h > cursor_area + 2 {
            self.pinned_mode = true;
            // Pin at current bottom: total - cursor_area (just above the cursor area)
            self.pinned_bottom_line = total.saturating_sub(cursor_area);
        }

        // Page size: scrollback area height - 1 (for context overlap)
        let scrollback_height = self.scrollback_area_height();
        let page = scrollback_height.saturating_sub(1).max(1);

        // Scroll up: move pinned_bottom_line up (toward older content)
        // Can't go below scrollback_height (need at least that many lines to show)
        let min_bottom = scrollback_height;
        self.pinned_bottom_line = self.pinned_bottom_line.saturating_sub(page).max(min_bottom);

        // scroll_offset tracks position for non-pinned mode; keep in sync
        self.scroll_offset = total.saturating_sub(self.pinned_bottom_line);

        self.state.mark_dirty();
        HandleResult::Consumed
    }

    pub(crate) fn scroll_down_page(&mut self) -> HandleResult {
        let cursor_area = self.cursor_area_lines as usize;
        let total = self.total_lines();

        let scrollback_height = self.scrollback_area_height();
        let page = scrollback_height.saturating_sub(1).max(1);

        if self.pinned_mode {
            // Scroll down: move pinned_bottom_line down (toward newer content)
            let new_bottom = self.pinned_bottom_line + page;

            // The maximum pinned_bottom_line is total - cursor_area (live position)
            let max_bottom = total.saturating_sub(cursor_area);

            if new_bottom >= max_bottom {
                // We've caught up to live - exit pinned mode
                self.pinned_mode = false;
                self.pinned_bottom_line = 0;
                self.scroll_offset = 0;
            } else {
                self.pinned_bottom_line = new_bottom;
                self.scroll_offset = total.saturating_sub(self.pinned_bottom_line);
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
        let total = self.total_lines();

        // Cursor area top line index
        let cursor_top = total.saturating_sub(cursor_area);

        // Gap is lines between pinned_bottom_line and cursor_top
        cursor_top.saturating_sub(self.pinned_bottom_line)
    }
}
