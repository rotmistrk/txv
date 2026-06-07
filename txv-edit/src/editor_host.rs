//! EditorHost — trait for app-specific integration.
//!
//! The host provides services that the editor itself doesn't own:
//! clipboard access, external command execution, completion, etc.

/// Trait implemented by the application embedding the editor.
pub trait EditorHost {
    /// Execute an ex command (":w", ":q", ":set ...", custom app commands).
    /// Returns true if handled.
    fn execute_command(&mut self, cmd: &str) -> bool;

    /// Get completions for ex command input.
    fn complete_command(&self, prefix: &str) -> Vec<String>;

    /// Read from system clipboard.
    fn clipboard_get(&self) -> Option<String>;

    /// Write to system clipboard.
    fn clipboard_set(&mut self, text: &str);

    /// Notify host that the buffer was modified (for dirty indicators, autosave, etc.).
    fn on_modified(&mut self) {}

    /// Notify host that cursor moved (for status bar updates, etc.).
    fn on_cursor_moved(&mut self, _line: usize, _col: usize) {}
}

/// No-op host for standalone/testing usage.
pub struct NullHost;

impl EditorHost for NullHost {
    fn execute_command(&mut self, _cmd: &str) -> bool {
        false
    }

    fn complete_command(&self, _prefix: &str) -> Vec<String> {
        Vec::new()
    }

    fn clipboard_get(&self) -> Option<String> {
        None
    }

    fn clipboard_set(&mut self, _text: &str) {}
}
