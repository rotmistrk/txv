//! Row — a terminal line with soft-wrap tracking.

use super::TCell;

/// A single terminal row: cells + whether it soft-wraps to the next row.
#[derive(Clone)]
pub(super) struct Row {
    pub(crate) cells: Vec<TCell>,
    /// True if this row continues on the next row (auto-wrapped at column limit).
    pub(crate) wrapped: bool,
}

impl Row {
    pub fn new(cols: usize) -> Self {
        Self {
            cells: vec![TCell::default(); cols],
            wrapped: false,
        }
    }

    pub fn from_cells(cells: Vec<TCell>) -> Self {
        Self { cells, wrapped: false }
    }

    #[cfg(test)]
    pub fn is_blank(&self) -> bool {
        self.cells.iter().all(|c| c.ch == ' ' || c.ch == '\0')
    }
}
