//! Completion infrastructure — trait for providing completions to input widgets.

/// A completion candidate. Implementors provide text, display label, and kind.
pub trait Completion {
    /// The text to insert when this completion is accepted.
    fn text(&self) -> &str;

    /// The display label shown in the popup.
    fn display(&self) -> &str;

    /// A category tag (e.g. "command", "file", "dir").
    fn kind(&self) -> &str;
}

/// Visitor callback type for completion results.
pub type CompletionVisitor<'a> = dyn FnMut(&dyn Completion) -> Result<bool, Box<dyn std::error::Error>> + 'a;

/// Trait for providing completions via visitor pattern.
/// Implementors call the visitor for each matching candidate.
pub trait Completer: Send {
    /// Provide completions for the given input at the given cursor position.
    /// Calls `visitor` for each candidate. Visitor returns `Ok(true)` to continue,
    /// `Ok(false)` to stop early, `Err` to abort.
    fn complete(
        &self,
        input: &str,
        cursor: usize,
        visitor: &mut CompletionVisitor<'_>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
