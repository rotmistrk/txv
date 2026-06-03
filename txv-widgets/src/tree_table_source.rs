//! TreeTableSource — data source trait for TreeTableView.

use txv_core::prelude::*;

/// Column alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColAlign {
    Left,
    Right,
    Center,
    /// Align at decimal point. Numbers without a dot are right-aligned to the dot position.
    Decimal,
}

/// Auto-detect alignment for a column by sampling cells.
/// If >50% of non-empty cells parse as numbers, returns Right (or Decimal if any have a dot).
pub fn auto_detect_align(cells: impl Iterator<Item = impl AsRef<str>>) -> ColAlign {
    let mut total = 0u32;
    let mut numeric = 0u32;
    let mut has_dot = false;
    for cell in cells {
        let s = cell.as_ref().trim();
        if s.is_empty() {
            continue;
        }
        total += 1;
        if s.parse::<f64>().is_ok() {
            numeric += 1;
            if s.contains('.') {
                has_dot = true;
            }
        }
    }
    if total == 0 || numeric * 2 <= total {
        return ColAlign::Left;
    }
    if has_dot {
        ColAlign::Decimal
    } else {
        ColAlign::Right
    }
}

/// Validator for cell editing. Presence = column is editable.
pub trait CellValidator: Send + Sync {
    fn validate(&self, text: &str) -> Result<(), String>;
}

/// Validator that accepts any input.
pub struct AcceptAll;

impl CellValidator for AcceptAll {
    fn validate(&self, _text: &str) -> Result<(), String> {
        Ok(())
    }
}

/// Data source for TreeTableView. Extends tree data with column cells.
pub trait TreeTableSource: Send + 'static {
    // Tree column
    fn visible_count(&self) -> usize;
    fn label(&self, row: usize) -> &str;
    fn depth(&self, row: usize) -> usize;
    fn is_expandable(&self, row: usize) -> bool;
    fn is_expanded(&self, row: usize) -> bool;
    fn toggle(&mut self, row: usize);
    fn style(&self, _row: usize) -> Style {
        Style::default()
    }
    fn highlight_positions(&self, _row: usize) -> Option<&[usize]> {
        None
    }
    fn filter_status(&self) -> Option<&str> {
        None
    }
    // Extra columns
    fn column_count(&self) -> usize;
    fn cell(&self, row: usize, col: usize) -> &str;
    fn cell_style(&self, _row: usize, _col: usize) -> Style {
        Style::default()
    }
    /// Return a validator for the column. None = not editable.
    /// col 0 = tree column, 1..N = extra columns.
    fn column_validator(&self, _col: usize) -> Option<&dyn CellValidator> {
        None
    }
    /// Alignment for extra column. Default: Left.
    /// If Decimal, the draw logic aligns cells at the '.' character.
    fn column_align(&self, _col: usize) -> ColAlign {
        ColAlign::Left
    }
    /// When true, the tree column skips indent and expand marker — label is printed at x=0.
    fn raw_labels(&self) -> bool {
        false
    }

    // --- Structural operations (return new cursor position or None) ---

    fn can_add_sibling(&self, _row: usize) -> bool {
        false
    }
    fn can_add_child(&self, _row: usize) -> bool {
        false
    }
    fn can_delete(&self, _row: usize) -> bool {
        false
    }
    fn can_swap_up(&self, _row: usize) -> bool {
        false
    }
    fn can_swap_down(&self, _row: usize) -> bool {
        false
    }
    fn can_promote(&self, _row: usize) -> bool {
        false
    }
    fn can_demote(&self, _row: usize) -> bool {
        false
    }

    fn add_sibling(&mut self, _row: usize) -> Option<usize> {
        None
    }
    fn add_child(&mut self, _row: usize) -> Option<usize> {
        None
    }
    fn delete(&mut self, _row: usize) -> Option<usize> {
        None
    }
    fn swap_up(&mut self, _row: usize) -> Option<usize> {
        None
    }
    fn swap_down(&mut self, _row: usize) -> Option<usize> {
        None
    }
    fn promote(&mut self, _row: usize) -> Option<usize> {
        None
    }
    fn demote(&mut self, _row: usize) -> Option<usize> {
        None
    }

    // --- Editing ---

    fn commit_edit(&mut self, _row: usize, _col: usize, _text: &str) {}

    // --- Undo support ---

    fn save_snapshot(&mut self) {}
    fn undo(&mut self) -> bool {
        false
    }
    fn redo(&mut self) -> bool {
        false
    }
}
