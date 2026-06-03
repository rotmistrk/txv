//! CompletionList — ListData for completions with separate text and display.

use txv_core::prelude::*;

use crate::list_view::ListData;

/// A completion item with insertion text and display label.
pub struct CompletionItem {
    text: String,
    display: String,
}

impl CompletionItem {
    pub fn new(text: String, display: String) -> Self {
        Self { text, display }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

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
        self.items.get(index).map(|i| i.text.as_str())
    }

    pub fn max_display_width(&self) -> usize {
        self.items.iter().map(|i| i.display.len()).max().unwrap_or(0)
    }
}

impl ListData for CompletionList {
    fn len(&self) -> usize {
        self.items.len()
    }

    fn label(&self, index: usize) -> &str {
        self.items.get(index).map(|i| i.display.as_str()).unwrap_or("")
    }

    fn style(&self, _index: usize) -> Style {
        txv_core::palette::palette().style(StyleId::PopupBackground)
    }
}
