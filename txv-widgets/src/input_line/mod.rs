//! InputLine — single-line text input with history, completion, and selection.

mod completion;
mod completion_item;
pub(crate) mod completion_source;
mod handle_key;
mod history;
mod readline;
mod selection;
#[cfg(test)]
mod tests;
mod view_impl;

use std::sync::Arc;

use txv_core::prelude::*;

/// Emitted by InputLine on Ctrl-C when selection exists. Data: `Box<String>`.
pub const CM_COPY_TO_CLIPBOARD: CommandId = 150;
/// Emitted by InputLine on Ctrl-V. App should respond with CM_CLIPBOARD_PASTE.
pub const CM_PASTE_REQUEST: CommandId = 151;
/// Sent to InputLine with text to insert. Data: `Box<String>`.
pub const CM_CLIPBOARD_PASTE: CommandId = 152;

pub struct InputLine {
    pub(crate) state: ViewState,
    pub(crate) text: String,
    /// Cursor position as char index.
    pub(crate) cursor: usize,
    /// Selection anchor (char index). Selection spans anchor..cursor.
    pub(crate) selection: Option<usize>,
    pub(crate) history: Vec<String>,
    pub(crate) history_pos: Option<usize>,
    pub(crate) shared_history: Option<txv_core::shared_history::SharedHistory>,
    pub(crate) completer: Option<Box<dyn Completer>>,
    pub(crate) submit_command: CommandId,
    /// Command emitted on every text change (None = silent).
    pub(crate) change_command: Option<CommandId>,
    /// Command ID that is allowed to set text content (prefill).
    pub(crate) prefill_command: Option<CommandId>,
    pub(crate) palette: Option<Arc<dyn Palette>>,
    /// Whether popup is currently visible.
    pub(crate) sidekick_visible: bool,
    /// Shared clipboard ring (direct access, no events needed).
    pub(crate) clipboard: Option<txv_core::clipboard_ring::ClipboardHandle>,
    /// When true, display chars as '*' (password mode).
    pub(crate) password: bool,
    /// When true, don't auto-resize bounds — scroll within fixed width instead.
    pub(crate) constrained: bool,
}

impl InputLine {
    pub fn new() -> Self {
        Self {
            state: ViewState::default(),
            text: String::new(),
            cursor: 0,
            selection: None,
            history: Vec::new(),
            history_pos: None,
            shared_history: None,
            completer: None,
            submit_command: CM_OK,
            change_command: None,
            prefill_command: None,
            palette: None,
            sidekick_visible: false,
            clipboard: None,
            password: false,
            constrained: false,
        }
    }

    pub fn with_command(mut self, id: CommandId) -> Self {
        self.submit_command = id;
        self
    }

    pub fn with_change_command(mut self, id: CommandId) -> Self {
        self.change_command = Some(id);
        self
    }

    pub fn with_prefill_command(mut self, id: CommandId) -> Self {
        self.prefill_command = Some(id);
        self
    }

    pub fn with_completer(mut self, c: Box<dyn Completer>) -> Self {
        self.completer = Some(c);
        self
    }

    pub fn with_history(mut self, h: txv_core::shared_history::SharedHistory) -> Self {
        self.shared_history = Some(h);
        self
    }

    pub fn with_clipboard(mut self, handle: txv_core::clipboard_ring::ClipboardHandle) -> Self {
        self.clipboard = Some(handle);
        self
    }

    pub fn with_password(mut self) -> Self {
        self.password = true;
        self
    }

    /// Enable constrained mode — don't auto-resize bounds, scroll within fixed width.
    /// Use this when embedding InputLine in a Group that manages its bounds.
    pub fn with_constrained(mut self) -> Self {
        self.constrained = true;
        self
    }

    pub fn set_completer(&mut self, c: Box<dyn Completer>) {
        self.completer = Some(c);
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor_pos(&self) -> usize {
        self.cursor
    }

    /// Push text to clipboard ring (or emit event as fallback).
    pub(crate) fn clipboard_copy(&mut self, text: &str) {
        if let Some(ref clip) = self.clipboard {
            if let Ok(mut ring) = clip.lock() {
                ring.push(text, "input");
                return;
            }
        }
        self.state
            .put_command(CM_COPY_TO_CLIPBOARD, Some(Box::new(text.to_string())));
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.cursor = self.text.chars().count();
        self.selection = None;
        self.update_width();
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.selection = None;
        self.update_width();
    }

    pub fn select_all(&mut self) {
        if !self.text.is_empty() {
            self.selection = Some(0);
            self.cursor = self.text.chars().count();
        }
        self.state.mark_dirty();
    }

    pub(crate) fn handle_char(&mut self, ch: char) {
        self.delete_selection();
        let byte_pos = self.char_to_byte(self.cursor);
        self.text.insert(byte_pos, ch);
        self.cursor += 1;
        self.update_width();
        self.notify_change();
    }

    pub(crate) fn handle_backspace(&mut self) {
        if self.selection.is_some() {
            self.delete_selection();
        } else if self.cursor > 0 {
            self.cursor -= 1;
            let byte_pos = self.char_to_byte(self.cursor);
            let next_byte = self.char_to_byte(self.cursor + 1);
            self.text.drain(byte_pos..next_byte);
            self.update_width();
        }
    }

    pub(crate) fn handle_delete(&mut self) {
        if self.selection.is_some() {
            self.delete_selection();
        } else if self.cursor < self.char_count() {
            let byte_pos = self.char_to_byte(self.cursor);
            let next_byte = self.char_to_byte(self.cursor + 1);
            self.text.drain(byte_pos..next_byte);
            self.update_width();
        }
    }

    pub(crate) fn resolve_style(&self, id: StyleId) -> Style {
        match &self.palette {
            Some(p) => p.style(id),
            None => txv_core::palette::palette().style(id),
        }
    }

    /// Emit change_command if configured.
    pub(crate) fn notify_change(&self) {
        if let Some(id) = self.change_command {
            self.state.put_command(id, None);
        }
    }
}
