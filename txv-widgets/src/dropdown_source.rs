//! DropdownSource trait — data provider for DropdownMenu.

use txv_core::prelude::Style;

/// Data source for DropdownMenu. Provides items with filtering.
pub trait DropdownSource: Send + 'static {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn label(&self, idx: usize) -> &str;
    /// Optional secondary text (right-aligned, dimmed).
    fn secondary(&self, _idx: usize) -> &str {
        ""
    }
    /// Optional badge (single char with style).
    fn badge(&self, _idx: usize) -> Option<(char, Style)> {
        None
    }
    /// Filter items by query. Called on every keystroke.
    fn filter(&mut self, query: &str);
    /// Number of visible items after filtering.
    fn visible_len(&self) -> usize;
    /// Map visible index to original index.
    fn visible_index(&self, visible_idx: usize) -> usize;
}
