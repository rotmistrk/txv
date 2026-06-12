//! DropdownSource for InputLine completion items.

use txv_core::prelude::Style;

use super::completion_item::CompletionItem;
use crate::dropdown_source::DropdownSource;

/// DropdownSource wrapping completion items.
pub(crate) struct CompletionSource {
    items: Vec<CompletionItem>,
}

impl CompletionSource {
    pub(crate) fn new(items: Vec<CompletionItem>) -> Self {
        Self { items }
    }

    pub(crate) fn text_at(&self, idx: usize) -> Option<&str> {
        self.items.get(idx).map(|i| i.text())
    }
}

impl DropdownSource for CompletionSource {
    fn len(&self) -> usize {
        self.items.len()
    }

    fn label(&self, idx: usize) -> &str {
        self.items.get(idx).map(|i| i.display()).unwrap_or("")
    }

    fn badge(&self, _idx: usize) -> Option<(&str, Style)> {
        None
    }
}
