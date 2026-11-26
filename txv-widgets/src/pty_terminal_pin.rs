//! Pinned-scroll positions for PtyTerminal.
//!
//! Two coordinate systems:
//! - **index**: position in the current `scrollback + grid` sequence
//!   (0 = oldest kept row). Shifts by one each time the full scrollback
//!   ring drops a row.
//! - **history**: `index + scrollback_evicted()`. Never shifts, so a pinned
//!   view stays on the same content while output keeps arriving.
//!
//! The pin is stored in history coordinates and converted on every use.

use crate::pty_terminal::PtyTerminal;

impl PtyTerminal {
    /// Rows in the top (frozen history) area while pinned.
    pub(crate) fn scrollback_area_height(&self) -> usize {
        let h = self.prev_rows as usize;
        let cursor_area = self.cursor_area_lines as usize;
        if self.pinned_mode && cursor_area > 0 {
            h.saturating_sub(cursor_area + 1)
        } else {
            h
        }
    }

    /// Whether the terminal is tall enough for the split pinned view.
    pub(crate) fn can_pin(&self) -> bool {
        let cursor_area = self.cursor_area_lines;
        cursor_area > 0 && self.prev_rows > cursor_area + 2
    }

    /// First grid row of the live area. The live area ends at the cursor
    /// row, so it shows the prompt even when the cursor is not at the
    /// bottom (for example after `clear`).
    pub(crate) fn live_start_row(&self) -> usize {
        let height = self.cursor_area_lines as usize;
        let grid = self.termbuf.grid_rows() as usize;
        let (_, cursor_y) = self.termbuf.cursor();
        let end = (cursor_y as usize + 1).min(grid).max(height.min(grid));
        end.saturating_sub(height)
    }

    /// Index of the first live-area row.
    pub(crate) fn live_top_index(&self) -> usize {
        self.termbuf.scrollback_len() + self.live_start_row()
    }

    /// Index of the row just below the frozen top area. Clamped so the top
    /// area is always full: if the pinned rows were evicted, the view shows
    /// the oldest rows still kept.
    pub(crate) fn pinned_bottom_index(&self) -> usize {
        let evicted = self.termbuf.scrollback_evicted();
        let index = self.pinned_bottom_hist.saturating_sub(evicted);
        let min = self.scrollback_area_height();
        index.min(self.live_top_index()).max(min)
    }

    pub(crate) fn set_pinned_bottom_index(&mut self, index: usize) {
        self.pinned_bottom_hist = index + self.termbuf.scrollback_evicted();
    }

    pub(crate) fn enter_pinned(&mut self) {
        self.pinned_mode = true;
        self.set_pinned_bottom_index(self.live_top_index());
    }

    pub(crate) fn exit_pinned(&mut self) {
        self.pinned_mode = false;
        self.pinned_bottom_hist = 0;
        self.scroll_offset = 0;
    }
}
