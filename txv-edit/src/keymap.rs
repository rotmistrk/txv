//! KeymapHandler — pluggable keybinding scheme interface.

use txv_core::prelude::*;

use crate::editor_core::EditorCore;

/// Result of a keymap handling a key event.
pub enum KeymapResult {
    /// Key was consumed, editor state may have changed.
    Consumed,
    /// Key was not handled — pass to host.
    Ignored,
    /// Execute an ex command (e.g., ":w", ":q").
    ExCommand(String),
}

/// Cursor shape hint for the terminal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorStyle {
    Block,
    Bar,
    Underline,
    Hidden,
}

/// Trait for pluggable keybinding handlers (vi, emacs, etc.).
pub trait KeymapHandler {
    /// Handle a key event, mutating editor state as needed.
    fn handle_key(&mut self, editor: &mut EditorCore, key: KeyEvent) -> KeymapResult;

    /// Current mode label for status display (e.g., "NORMAL", "INSERT", "VISUAL").
    fn mode_label(&self) -> &str;

    /// Desired cursor shape for the current mode.
    fn cursor_style(&self) -> CursorStyle;

    /// Whether the editor is in insert/append mode (affects character echo).
    fn is_insert_mode(&self) -> bool;
}
