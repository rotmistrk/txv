//! DropdownSource trait — data provider for DropdownMenu.

use txv_core::prelude::Style;

/// Data source for DropdownMenu. Provides all items; filtering is done by the widget.
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
}
