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
        // Page keys work even when exited (for reviewing scrollback)
        if key.code() == KeyCode::PageUp {
            return self.scroll_up_page();
        }
        if key.code() == KeyCode::PageDown {
            return self.scroll_down_page();
        }
        // Any other key exits scrollback/pinned mode
        if self.scroll_offset > 0 || self.pinned_mode {
            self.exit_pinned();
            self.state.mark_dirty();
        }
        // Don't send keys to exited terminal
        if self.exited {
            return HandleResult::Consumed;
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

    pub(crate) fn scroll_up_page(&mut self) -> HandleResult {
        if !self.pinned_mode && self.can_pin() {
            self.enter_pinned();
        }
        let scrollback_height = self.scrollback_area_height();
        let page = scrollback_height.saturating_sub(1).max(1);
        if self.pinned_mode {
            // Toward older content; the top area must stay full.
            let index = self.pinned_bottom_index().saturating_sub(page).max(scrollback_height);
            self.set_pinned_bottom_index(index);
            self.scroll_offset = self.total_lines().saturating_sub(index);
        } else {
            let max = self.termbuf.scrollback_len();
            self.scroll_offset = (self.scroll_offset + page).min(max);
        }
        self.state.mark_dirty();
        HandleResult::Consumed
    }

    pub(crate) fn scroll_down_page(&mut self) -> HandleResult {
        let scrollback_height = self.scrollback_area_height();
        let page = scrollback_height.saturating_sub(1).max(1);
        if self.pinned_mode {
            let index = self.pinned_bottom_index() + page;
            if index >= self.live_top_index() {
                // Caught up with the live area.
                self.exit_pinned();
            } else {
                self.set_pinned_bottom_index(index);
                self.scroll_offset = self.total_lines().saturating_sub(index);
            }
        } else {
            self.scroll_offset = self.scroll_offset.saturating_sub(page);
        }
        self.state.mark_dirty();
        HandleResult::Consumed
    }

    /// Calculate the displayed gap: logical lines between scrollback bottom and cursor area top.
    /// Accounts for line wrapping - wrapped rows count as 1 logical line together.
    pub(crate) fn calculate_displayed_gap(&self) -> usize {
        if !self.pinned_mode {
            return 0;
        }
        let sb_len = self.termbuf.scrollback_len();
        let gap_start = self.pinned_bottom_index();
        let gap_end = self.live_top_index();

        if gap_start >= gap_end {
            return 0;
        }

        // Count logical lines: a row starts a new logical line if the previous row
        // was NOT wrapped. First row always starts a logical line.
        let mut logical_lines = 0;
        for line_idx in gap_start..gap_end {
            let prev_wrapped = if line_idx == 0 {
                false
            } else {
                self.is_line_wrapped(line_idx - 1, sb_len)
            };
            if !prev_wrapped {
                logical_lines += 1;
            }
        }
        logical_lines
    }

    /// Check if a line (by absolute index) is wrapped.
    fn is_line_wrapped(&self, line_idx: usize, sb_len: usize) -> bool {
        if line_idx < sb_len {
            self.termbuf.scrollback_wrapped(sb_len - 1 - line_idx)
        } else {
            self.termbuf.grid_wrapped(line_idx - sb_len)
        }
    }
}
