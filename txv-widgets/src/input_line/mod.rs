//! InputLine — single-line text input with history, completion, and selection.

mod completion;
mod completion_frame;
mod completion_list;
mod handle_key;
mod history;
mod readline;
#[cfg(test)]
mod tests;
mod view_impl;

use std::sync::{Arc, Mutex};

use txv_core::prelude::*;

use crate::list_view::ListView;
pub use completion_list::CompletionList;

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
    pub(crate) completer: Option<Box<dyn Completer>>,
    pub(crate) submit_command: CommandId,
    pub(crate) palette: Option<Arc<dyn Palette>>,
    /// Shared completion popup (ListView held by SidekickManager).
    pub(crate) popup: Arc<Mutex<ListView<CompletionList>>>,
    /// Framed wrapper sent to SidekickManager.
    pub(crate) popup_frame: Arc<Mutex<completion_frame::CompletionFrame>>,
    /// Whether popup is currently visible.
    pub(crate) sidekick_visible: bool,
}

impl InputLine {
    pub fn new() -> Self {
        let list = ListView::new(CompletionList::new(Vec::new()));
        let popup = Arc::new(Mutex::new(list));
        let frame = completion_frame::CompletionFrame::new(Arc::clone(&popup));
        Self {
            state: ViewState::default(),
            text: String::new(),
            cursor: 0,
            selection: None,
            history: Vec::new(),
            history_pos: None,
            completer: None,
            submit_command: CM_OK,
            palette: None,
            popup,
            popup_frame: Arc::new(Mutex::new(frame)),
            sidekick_visible: false,
        }
    }

    pub fn with_command(mut self, id: CommandId) -> Self {
        self.submit_command = id;
        self
    }

    pub fn with_completer(mut self, c: Box<dyn Completer>) -> Self {
        self.completer = Some(c);
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

    pub(crate) fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    pub(crate) fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection.map(|anchor| {
            let lo = anchor.min(self.cursor);
            let hi = anchor.max(self.cursor);
            (lo, hi)
        })
    }

    /// Get the currently selected text, if any.
    pub fn selected_text(&self) -> Option<String> {
        let (lo, hi) = self.selection_range()?;
        let byte_lo = self.char_to_byte(lo);
        let byte_hi = self.char_to_byte(hi);
        Some(self.text[byte_lo..byte_hi].to_string())
    }

    /// Insert text at cursor, replacing selection if active.
    pub fn insert_text(&mut self, text: &str) {
        if self.selection.is_some() {
            self.delete_selection();
        }
        let byte_pos = self.char_to_byte(self.cursor);
        self.text.insert_str(byte_pos, text);
        self.cursor += text.chars().count();
        self.update_width();
        self.state.mark_dirty();
    }

    pub(crate) fn delete_selection(&mut self) {
        if let Some((lo, hi)) = self.selection_range() {
            let byte_lo = self.char_to_byte(lo);
            let byte_hi = self.char_to_byte(hi);
            self.text.drain(byte_lo..byte_hi);
            self.cursor = lo;
            self.selection = None;
            self.update_width();
        }
    }

    /// Convert char index to byte offset.
    fn char_to_byte(&self, char_idx: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_idx)
            .map(|(b, _)| b)
            .unwrap_or(self.text.len())
    }

    /// Auto-resize bounds to fit text (standalone mode).
    fn update_width(&mut self) {
        self.state.mark_dirty();
        let w = (self.char_count() as u16).saturating_add(2).max(10);
        let b = self.state.bounds();
        if b.w != w {
            self.state.set_bounds(Rect::new(b.x, b.y, w, 1));
        }
    }

    pub(crate) fn handle_char(&mut self, ch: char) {
        self.delete_selection();
        let byte_pos = self.char_to_byte(self.cursor);
        self.text.insert(byte_pos, ch);
        self.cursor += 1;
        self.update_width();
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

    pub(crate) fn handle_nav(&mut self, shift: bool, new_cursor: usize) {
        if shift {
            if self.selection.is_none() {
                self.selection = Some(self.cursor);
            }
        } else {
            self.selection = None;
        }
        self.cursor = new_cursor;
        self.state.mark_dirty();
    }

    pub(crate) fn handle_command(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        if let Some(text) = data.as_ref().and_then(|d| d.downcast_ref::<String>()) {
            self.set_text(text);
            return HandleResult::Consumed;
        }
        HandleResult::Ignored
    }

    pub(crate) fn resolve_style(&self, id: StyleId) -> Style {
        match &self.palette {
            Some(p) => p.style(id),
            None => txv_core::palette::palette().style(id),
        }
    }

    pub(crate) fn visible_start(&self, width: usize) -> usize {
        if width == 0 {
            return 0;
        }
        let total = self.char_count();
        let mut start = if self.cursor >= width {
            self.cursor - width + 1
        } else {
            0
        };
        // If cursor lands on the last cell and there's text to the right,
        // scroll one more so the cursor isn't on the right-overflow '…' position.
        let cursor_pos = self.cursor - start;
        if cursor_pos == width - 1 && start + width < total {
            start += 1;
        }
        // If cursor lands on position 0 and there's left overflow,
        // scroll one less so the cursor isn't on the left-overflow '…' position.
        if start > 0 && self.cursor == start {
            start -= 1;
        }
        start
    }
}
