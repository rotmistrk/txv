//! CompletionItem — a single completion entry.

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

    pub fn display(&self) -> &str {
        &self.display
    }
}
