//! CompletionList — ListData for completions with separate text and display.

use txv_core::palette::palette;
use txv_core::prelude::*;

use super::completion_item::CompletionItem;
use crate::list_view::ListData;

/// ListData for completions. Shows `display`, applies `text`.
pub struct CompletionList {
    items: Vec<CompletionItem>,
}

impl CompletionList {
    pub fn new(items: Vec<CompletionItem>) -> Self {
        Self { items }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn selected_text(&self, index: usize) -> Option<&str> {
        self.items.get(index).map(|i| i.text())
    }

    pub fn max_display_width(&self) -> usize {
        self.items.iter().map(|i| i.display().len()).max().unwrap_or(0)
    }
}

impl ListData for CompletionList {
    fn len(&self) -> usize {
        self.items.len()
    }

    fn label(&self, index: usize) -> &str {
        self.items.get(index).map(|i| i.display()).unwrap_or("")
    }

    fn style(&self, _index: usize) -> Style {
        palette().style(StyleId::PopupBackground)
    }
}
