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
        if !self.pinned_mode && cursor_area > 0 && h > cursor_area + 1 {
            self.pinned_mode = true;
            self.gap = 0;
            // scroll_offset stays at 0, we'll scroll from current position
        }

        let max = self.termbuf.scrollback_len() + self.gap;
        let page = h.saturating_sub(1).max(1);
        self.scroll_offset = (self.scroll_offset + page).min(max);
        self.state.mark_dirty();
        HandleResult::Consumed
    }

    pub(crate) fn scroll_down_page(&mut self) -> HandleResult {
        let h = self.prev_rows as usize;
        let page = h.saturating_sub(1).max(1);
        let new_offset = self.scroll_offset.saturating_sub(page);

        if self.pinned_mode {
            // In pinned mode, scrolling down reduces gap first, then scroll_offset
            if new_offset == 0 && self.gap == 0 {
                // Exiting pinned mode - back to live view
                self.pinned_mode = false;
                self.scroll_offset = 0;
            } else if new_offset < self.scroll_offset {
                // Consume gap before reducing scroll_offset
                let consumed = self.scroll_offset - new_offset;
                if self.gap >= consumed {
                    self.gap -= consumed;
                } else {
                    self.gap = 0;
                }
                self.scroll_offset = new_offset;
            }
        } else {
            self.scroll_offset = new_offset;
        }

        self.state.mark_dirty();
        HandleResult::Consumed
    }
}
