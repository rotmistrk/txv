//! CSI dispatch helper methods for Performer.

use super::row::Row;
use super::vte_handler::Performer;
use super::TCell;

impl Performer<'_> {
    pub(super) fn csi_cursor_up(&mut self, n: u16) {
        let n = if n == 0 {
            1
        } else {
            n
        };
        *self.cursor_y = self.cursor_y.saturating_sub(n);
    }

    pub(super) fn csi_cursor_down(&mut self, n: u16) {
        let n = if n == 0 {
            1
        } else {
            n
        };
        *self.cursor_y = (*self.cursor_y + n).min(self.rows.saturating_sub(1));
    }

    pub(super) fn csi_cursor_forward(&mut self, n: u16) {
        let n = if n == 0 {
            1
        } else {
            n
        };
        *self.cursor_x = (*self.cursor_x + n).min(self.cols.saturating_sub(1));
    }

    pub(super) fn csi_cursor_back(&mut self, n: u16) {
        let n = if n == 0 {
            1
        } else {
            n
        };
        *self.cursor_x = self.cursor_x.saturating_sub(n);
    }

    pub(super) fn csi_cursor_position(&mut self, p1: u16, ps: &[u16]) {
        let row = if p1 == 0 {
            1
        } else {
            p1
        };
        let col = ps.get(1).copied().unwrap_or(1).max(1);
        *self.cursor_y = (row - 1).min(self.rows.saturating_sub(1));
        *self.cursor_x = (col - 1).min(self.cols.saturating_sub(1));
    }

    pub(super) fn csi_cursor_col(&mut self, p1: u16) {
        let col = if p1 == 0 {
            1
        } else {
            p1
        };
        *self.cursor_x = (col - 1).min(self.cols.saturating_sub(1));
    }

    pub(super) fn csi_insert_lines(&mut self, p1: u16) {
        let n = if p1 == 0 {
            1
        } else {
            p1 as usize
        };
        let y = *self.cursor_y as usize;
        let bot = *self.scroll_bottom as usize;
        for _ in 0..n {
            if y <= bot && bot < self.cells.len() {
                self.cells.remove(bot);
                self.cells.insert(y, Row::new(self.cols as usize));
            }
        }
    }

    pub(super) fn csi_delete_lines(&mut self, p1: u16) {
        let n = if p1 == 0 {
            1
        } else {
            p1 as usize
        };
        let y = *self.cursor_y as usize;
        let bot = *self.scroll_bottom as usize;
        for _ in 0..n {
            if y <= bot && bot < self.cells.len() {
                self.cells.remove(y);
                self.cells.insert(bot, Row::new(self.cols as usize));
            }
        }
    }

    pub(super) fn csi_scroll_up(&mut self, p1: u16) {
        let n = if p1 == 0 {
            1
        } else {
            p1
        };
        for _ in 0..n {
            self.scroll_up();
        }
    }

    pub(super) fn csi_scroll_down(&mut self, p1: u16) {
        let n = if p1 == 0 {
            1
        } else {
            p1
        };
        for _ in 0..n {
            self.scroll_down();
        }
    }

    pub(super) fn csi_sgr(&mut self, ps: &[u16]) {
        if ps.is_empty() {
            self.set_sgr(&[0]);
        } else {
            self.set_sgr(ps);
        }
    }

    pub(super) fn csi_set_scroll_region(&mut self, p1: u16, ps: &[u16]) {
        let top = if p1 == 0 {
            1
        } else {
            p1
        };
        let bot = ps.get(1).copied().unwrap_or(self.rows).min(self.rows);
        *self.scroll_top = top.saturating_sub(1);
        *self.scroll_bottom = bot.saturating_sub(1);
        *self.cursor_x = 0;
        *self.cursor_y = 0;
    }

    pub(super) fn csi_delete_chars(&mut self, p1: u16) {
        let n = if p1 == 0 {
            1
        } else {
            p1 as usize
        };
        let y = *self.cursor_y as usize;
        let x = *self.cursor_x as usize;
        if y >= self.rows as usize {
            return;
        }
        let row = &mut self.cells[y].cells;
        let end = (x + n).min(row.len());
        row.drain(x..end);
        row.resize(self.cols as usize, TCell::default());
    }

    pub(super) fn csi_insert_chars(&mut self, p1: u16) {
        let n = if p1 == 0 {
            1
        } else {
            p1 as usize
        };
        let y = *self.cursor_y as usize;
        let x = *self.cursor_x as usize;
        if y >= self.rows as usize {
            return;
        }
        let row = &mut self.cells[y].cells;
        for _ in 0..n {
            if x < row.len() {
                row.insert(x, TCell::default());
            }
        }
        row.truncate(self.cols as usize);
    }
}
