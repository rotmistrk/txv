//! Terminal resize logic — simple and reflow modes.

use super::row::Row;
use super::{TCell, TermBuf};

impl TermBuf {
    /// Resize for live PTY: reflow scrollback + grid above cursor, clear cursor row (shell redraws).
    pub fn resize_simple(&mut self, cols: u16, rows: u16) {
        let new_cols = cols as usize;
        let new_rows = rows as usize;
        let cursor_y = self.cursor_y as usize;

        // Collect scrollback + grid rows above cursor for reflow
        let mut content_rows = self.scrollback.drain_all();
        let above = cursor_y.min(self.cells.len());
        content_rows.extend(self.cells.drain(..above));

        // Reflow all historical content to new width
        let reflowed = Self::reflow_rows(content_rows, new_cols);

        // Split reflowed into what fits in grid vs scrollback
        let grid_slots = new_rows.saturating_sub(1); // leave 1 row for cursor
        let sb_end = reflowed.len().saturating_sub(grid_slots);
        let sb_rows: Vec<Row> = reflowed[..sb_end].to_vec();
        let grid_above: Vec<Row> = reflowed[sb_end..].to_vec();

        self.scrollback.replace(sb_rows);

        // Build new grid: reflowed content + blank cursor row + blank below
        let mut new_cells: Vec<Row> = Vec::with_capacity(new_rows);
        new_cells.extend(grid_above);
        let new_cursor_y = new_cells.len();
        while new_cells.len() < new_rows {
            new_cells.push(Row::new(new_cols));
        }

        self.cells = new_cells;
        self.cols = cols;
        self.rows = rows;
        self.scroll_bottom = rows.saturating_sub(1);
        self.cursor_x = 0;
        self.cursor_y = (new_cursor_y as u16).min(rows.saturating_sub(1));
    }

    /// Resize the terminal buffer with reflow of soft-wrapped lines.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        let new_cols = cols as usize;
        let new_rows = rows as usize;
        let cursor_y = self.cursor_y as usize;

        // Collect content rows: scrollback + grid rows up to cursor (inclusive)
        let mut content_rows = self.scrollback.drain_all();
        let grid_content_end = (cursor_y + 1).min(self.cells.len());
        content_rows.extend(self.cells.drain(..grid_content_end));

        // Join wrapped rows into logical lines, then re-wrap to new width
        let reflowed = Self::reflow_rows(content_rows, new_cols);

        // Split into new scrollback and new grid
        let total = reflowed.len();
        let grid_start = total.saturating_sub(new_rows);
        let sb_rows: Vec<Row> = reflowed[..grid_start].to_vec();
        let mut grid_rows: Vec<Row> = reflowed[grid_start..].to_vec();
        let new_cursor_y = grid_rows.len().saturating_sub(1);

        // Pad grid to new_rows
        while grid_rows.len() < new_rows {
            grid_rows.push(Row::new(new_cols));
        }

        self.scrollback.replace(sb_rows);
        self.cells = grid_rows;
        self.cols = cols;
        self.rows = rows;
        self.scroll_bottom = rows.saturating_sub(1);
        self.cursor_y = (new_cursor_y as u16).min(rows.saturating_sub(1));
        self.cursor_x = self.cursor_x.min(cols.saturating_sub(1));
    }

    /// Join consecutive wrapped rows into logical lines, then re-wrap to new_cols.
    fn reflow_rows(rows: Vec<Row>, new_cols: usize) -> Vec<Row> {
        let mut result: Vec<Row> = Vec::new();
        let mut i = 0;
        while i < rows.len() {
            let mut logical: Vec<TCell> = Vec::new();
            loop {
                let row = &rows[i];
                logical.extend_from_slice(&row.cells);
                i += 1;
                if !row.wrapped || i >= rows.len() {
                    break;
                }
            }
            // Trim trailing blanks
            while logical.last().is_some_and(|c| c.ch == ' ' || c.ch == '\0') {
                logical.pop();
            }
            if logical.is_empty() {
                result.push(Row::new(new_cols));
            } else {
                let chunk_count = logical.len().div_ceil(new_cols);
                for (idx, chunk) in logical.chunks(new_cols).enumerate() {
                    let mut cells = chunk.to_vec();
                    cells.resize(new_cols, TCell::default());
                    let mut row = Row::from_cells(cells);
                    row.wrapped = idx + 1 < chunk_count;
                    result.push(row);
                }
            }
        }
        result
    }
}
