//! Editor accessor methods — getters and setters.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::highlight_state::HighlightState;
use super::keymap_vim::VimKeymap;
use super::options::EditorOptions;
use super::Editor;
use crate::buffer::PieceTable;
use crate::editor::ephemeral::EphemeralHighlights;
use crate::shared_register::RegisterHandle;
use txv_core::clipboard_ring::ClipboardHandle;

impl Editor {
    pub fn set_shared_state(&mut self, register: RegisterHandle, clipboard: ClipboardHandle) {
        self.shared_register = register;
        self.clipboard = Some(clipboard);
    }
    pub fn buffer_arc(&self) -> Arc<Mutex<PieceTable>> {
        self.buffer.clone()
    }
    pub fn search_pattern(&self) -> &str {
        &self.search_pattern
    }
    pub fn set_search_pattern(&mut self, pat: impl Into<String>) {
        self.search_pattern = pat.into();
    }
    pub fn visual_anchor(&self) -> Option<(usize, usize)> {
        self.visual_anchor
    }
    pub fn set_visual_anchor(&mut self, v: Option<(usize, usize)>) {
        self.visual_anchor = v;
    }
    pub fn viewport_scroll(&self) -> usize {
        self.viewport_scroll
    }
    pub fn set_viewport_scroll(&mut self, v: usize) {
        self.viewport_scroll = v;
    }
    pub fn viewport_height(&self) -> usize {
        self.viewport_height
    }
    pub fn viewport_width(&self) -> usize {
        self.viewport_width
    }
    pub fn h_scroll(&self) -> usize {
        self.h_scroll
    }
    pub fn set_h_scroll(&mut self, v: usize) {
        self.h_scroll = v;
    }
    pub fn options(&self) -> &EditorOptions {
        &self.options
    }
    pub fn options_mut(&mut self) -> &mut EditorOptions {
        &mut self.options
    }
    pub fn command_history(&self) -> &[String] {
        &self.command_history
    }
    pub fn command_history_mut(&mut self) -> &mut Vec<String> {
        &mut self.command_history
    }
    pub fn history_index(&self) -> Option<usize> {
        self.history_index
    }
    pub fn set_history_index(&mut self, v: Option<usize>) {
        self.history_index = v;
    }
    pub fn history_prefix(&self) -> &str {
        &self.history_prefix
    }
    pub fn set_history_prefix(&mut self, v: String) {
        self.history_prefix = v;
    }
    pub fn highlight(&self) -> Option<&HighlightState> {
        self.highlight.as_ref()
    }
    pub fn set_highlight(&mut self, v: Option<HighlightState>) {
        self.highlight = v;
    }
    pub fn take_highlight(&mut self) -> Option<HighlightState> {
        self.highlight.take()
    }
    pub fn ephemeral(&self) -> &EphemeralHighlights {
        &self.ephemeral
    }
    pub fn ephemeral_mut(&mut self) -> &mut EphemeralHighlights {
        &mut self.ephemeral
    }
    pub fn set_status(&mut self, v: String) {
        self.status = v;
    }
    pub fn incsearch_origin(&self) -> Option<(usize, usize)> {
        self.incsearch_origin
    }
    pub fn set_incsearch_origin(&mut self, v: Option<(usize, usize)>) {
        self.incsearch_origin = v;
    }
    pub fn keymap(&self) -> &VimKeymap {
        &self.keymap
    }
    pub fn keymap_mut(&mut self) -> &mut VimKeymap {
        &mut self.keymap
    }
    pub fn marks(&self) -> &HashMap<char, (usize, usize)> {
        &self.marks
    }
    pub fn marks_mut(&mut self) -> &mut HashMap<char, (usize, usize)> {
        &mut self.marks
    }
}
