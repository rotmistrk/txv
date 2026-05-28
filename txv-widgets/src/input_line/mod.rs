//! InputLine — single-line text input with history, completion, and selection.

mod view_impl;

use std::sync::Arc;

use txv_core::prelude::*;

pub struct InputLine {
    pub(crate) state: ViewState,
    pub(crate) text: String,
    pub(crate) cursor: usize,
    /// Selection anchor. When Some, text between anchor and cursor is selected.
    pub(crate) selection: Option<usize>,
    pub(crate) history: Vec<String>,
    pub(crate) history_pos: Option<usize>,
    pub(crate) completer: Option<Box<dyn Completer>>,
    pub(crate) submit_command: CommandId,
    pub(crate) palette: Option<Arc<dyn Palette>>,
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
            completer: None,
            submit_command: CM_OK,
            palette: None,
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
        self.cursor = self.text.len();
        self.selection = None;
        self.update_width();
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.selection = None;
        self.update_width();
    }

    /// Select all text. Typing replaces selection; nav deselects.
    pub fn select_all(&mut self) {
        if !self.text.is_empty() {
            self.selection = Some(0);
            self.cursor = self.text.len();
        }
        self.state.mark_dirty();
    }

    /// Returns (start, end) of selection range, or None.
    pub(crate) fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection.map(|anchor| {
            let lo = anchor.min(self.cursor);
            let hi = anchor.max(self.cursor);
            (lo, hi)
        })
    }

    /// Delete selected text, place cursor at start of selection.
    pub(crate) fn delete_selection(&mut self) {
        if let Some((lo, hi)) = self.selection_range() {
            self.text.drain(lo..hi);
            self.cursor = lo;
            self.selection = None;
            self.update_width();
        }
    }

    pub(crate) fn update_width(&mut self) {
        let w = (self.text.len() as u16).saturating_add(2).max(10);
        let b = self.state.bounds();
        if b.w != w {
            self.state.set_bounds(Rect::new(b.x, b.y, w, 1));
        }
        self.state.mark_dirty();
    }

    pub(crate) fn push_history(&mut self) {
        if !self.text.is_empty() {
            self.history.push(self.text.clone());
        }
        self.history_pos = None;
    }

    pub(crate) fn handle_char(&mut self, ch: char) {
        self.delete_selection();
        self.text.insert(self.cursor, ch);
        self.cursor += 1;
        self.update_width();
    }

    pub(crate) fn handle_backspace(&mut self) {
        if self.selection.is_some() {
            self.delete_selection();
        } else if self.cursor > 0 {
            self.cursor -= 1;
            self.text.remove(self.cursor);
            self.update_width();
        }
    }

    pub(crate) fn handle_delete(&mut self) {
        if self.selection.is_some() {
            self.delete_selection();
        } else if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
            self.update_width();
        }
    }

    pub(crate) fn handle_history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let pos = match self.history_pos {
            Some(p) => p.saturating_sub(1),
            None => self.history.len() - 1,
        };
        self.history_pos = Some(pos);
        self.text = self.history[pos].clone();
        self.cursor = self.text.len();
        self.selection = None;
        self.update_width();
    }

    pub(crate) fn handle_history_down(&mut self) {
        let Some(pos) = self.history_pos else {
            return;
        };
        if pos + 1 < self.history.len() {
            self.history_pos = Some(pos + 1);
            self.text = self.history[pos + 1].clone();
        } else {
            self.history_pos = None;
            self.text.clear();
        }
        self.cursor = self.text.len();
        self.selection = None;
        self.update_width();
    }

    pub(crate) fn handle_command(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        if let Some(text) = data.as_ref().and_then(|d| d.downcast_ref::<String>()) {
            self.set_text(text);
            return HandleResult::Consumed;
        }
        HandleResult::Ignored
    }

    pub(crate) fn try_complete(&mut self) {
        let Some(ref completer) = self.completer else {
            return;
        };
        let mut first: Option<String> = None;
        let mut count = 0u32;
        let _ = completer.complete(&self.text, self.cursor, &mut |c| {
            count += 1;
            if count == 1 {
                first = Some(c.text().to_string());
            }
            Ok(count < 2)
        });
        if count == 1 {
            if let Some(text) = first {
                self.text = text;
                self.cursor = self.text.len();
                self.update_width();
            }
        }
    }

    pub(crate) fn resolve_style(&self, id: StyleId) -> Style {
        match &self.palette {
            Some(p) => p.style(id),
            None => txv_core::palette::palette().style(id),
        }
    }

    pub(crate) fn visible_start(&self, width: usize) -> usize {
        if self.cursor >= width {
            self.cursor - width + 1
        } else {
            0
        }
    }
}

impl Default for InputLine {
    fn default() -> Self {
        Self::new()
    }
}
