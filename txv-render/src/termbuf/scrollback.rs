//! Scrollback ring buffer — stores rows pushed off the top of the terminal.

use std::collections::VecDeque;

use super::row::Row;

/// A capped ring buffer of terminal rows for scrollback history.
pub(super) struct Scrollback {
    lines: VecDeque<Row>,
    limit: usize,
}

impl Scrollback {
    pub fn new(limit: usize) -> Self {
        Self {
            lines: VecDeque::new(),
            limit,
        }
    }

    pub fn push(&mut self, row: Row) {
        if self.limit == 0 {
            return;
        }
        if self.lines.len() >= self.limit {
            self.lines.pop_front();
        }
        self.lines.push_back(row);
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn line_from_bottom(&self, offset: usize) -> Option<&Row> {
        if offset >= self.lines.len() {
            return None;
        }
        let idx = self.lines.len() - 1 - offset;
        self.lines.get(idx)
    }

    /// Drain all rows out (for reflow).
    pub fn drain_all(&mut self) -> Vec<Row> {
        self.lines.drain(..).collect()
    }

    /// Replace all rows (after reflow).
    pub fn replace(&mut self, rows: Vec<Row>) {
        self.lines = VecDeque::from(rows);
        // Trim to limit
        while self.lines.len() > self.limit {
            self.lines.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::termbuf::TCell;

    fn make_row(ch: char, width: usize) -> Row {
        Row::from_cells(vec![TCell { ch, ..TCell::default() }; width])
    }

    #[test]
    fn push_and_retrieve() {
        let mut sb = Scrollback::new(10);
        sb.push(make_row('A', 5));
        sb.push(make_row('B', 5));
        assert_eq!(sb.len(), 2);
        assert_eq!(sb.line_from_bottom(0).map(|r| r.cells[0].ch), Some('B'));
        assert_eq!(sb.line_from_bottom(1).map(|r| r.cells[0].ch), Some('A'));
    }

    #[test]
    fn respects_limit() {
        let mut sb = Scrollback::new(3);
        for ch in ['A', 'B', 'C', 'D', 'E'] {
            sb.push(make_row(ch, 5));
        }
        assert_eq!(sb.len(), 3);
        assert_eq!(sb.line_from_bottom(2).map(|r| r.cells[0].ch), Some('C'));
        assert_eq!(sb.line_from_bottom(0).map(|r| r.cells[0].ch), Some('E'));
    }

    #[test]
    fn zero_limit_stores_nothing() {
        let mut sb = Scrollback::new(0);
        sb.push(make_row('X', 5));
        assert_eq!(sb.len(), 0);
    }

    #[test]
    fn out_of_bounds_returns_none() {
        let mut sb = Scrollback::new(10);
        sb.push(make_row('A', 5));
        assert!(sb.line_from_bottom(1).is_none());
    }
}
