//! Completion infrastructure — trait for providing completions to input widgets.

/// A single completion candidate.
pub struct Completion {
    text: String,
    display: String,
    kind: &'static str,
}

impl Completion {
    pub fn new(text: String, display: String, kind: &'static str) -> Self {
        Self { text, display, kind }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn display(&self) -> &str {
        &self.display
    }

    pub fn kind(&self) -> &'static str {
        self.kind
    }
}

/// Trait for providing completions. Implemented by application-level completers.
pub trait Completer: Send {
    /// Return completions for the given input at the given cursor position.
    fn complete(&self, input: &str, cursor: usize) -> Vec<Completion>;
}
