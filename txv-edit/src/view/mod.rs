//! EditorView — a reusable View wrapping txv_edit::editor::Editor.
//!
//! Uses GroupState to host an InputLine for command/search mode.
//! Child 0 (when present): InputLine for `:` or `/` input.

mod cursor;
pub mod delegate;
pub mod draw;
mod handle;
mod handle_cmdline;
mod highlight_range;
mod line_decoration;

use std::path::{Path, PathBuf};

use txv_core::prelude::*;

pub use delegate::{EditorViewDelegate, NullDelegate};

use crate::editor::keymap::EditorMode;
use crate::editor::Editor;
use crate::highlight::{extension_from_path, HighlightCache, Highlighter};

/// Command IDs emitted by EditorView.
pub const CM_EDITOR_SAVE: u16 = 180;
pub const CM_EDITOR_CLOSE: u16 = 181;
pub const CM_EDITOR_CURSOR_MOVED: u16 = 182;
pub const CM_EDITOR_CONTENT_CHANGED: u16 = 183;
/// Internal: InputLine text changed (for incremental search).
pub const CM_CMDLINE_CHANGED: u16 = 184;

/// A reusable editor View. Parameterized by a delegate for app extensions.
pub struct EditorView<D: EditorViewDelegate = NullDelegate> {
    group: GroupState,
    editor: Editor,
    path: PathBuf,
    highlighter: Highlighter,
    hl_cache: HighlightCache,
    delegate: D,
    /// True when the InputLine child is present (command/search mode).
    cmdline_active: bool,
    /// Prefix char for the command line (':' or '/' or '?').
    cmdline_prefix: char,
    /// Number of search matches (updated during incsearch).
    match_count: usize,
    /// Tick counter for delegate.on_tick().
    tick_count: u64,
}

impl Default for EditorView<NullDelegate> {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorView<NullDelegate> {
    /// Create an empty editor view.
    pub fn new() -> Self {
        Self::with_delegate(NullDelegate)
    }

    /// Create from text content.
    pub fn from_text(content: &str) -> Self {
        let mut v = Self::new();
        v.editor.replace_content(content);
        v
    }

    /// Open a file. Returns io::Error if file cannot be read.
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let editor = Editor::open(path)?;
        let ext = extension_from_path(path).to_string();
        let mut view = Self {
            group: GroupState::new(ViewOptions::default().with_focusable()),
            editor,
            path: path.to_path_buf(),
            highlighter: Highlighter::new(),
            hl_cache: HighlightCache::new(&ext),
            delegate: NullDelegate,
            cmdline_active: false,
            cmdline_prefix: ':',
            match_count: 0,
            tick_count: 0,
        };
        view.group.set_title(file_title(path));
        Ok(view)
    }
}

impl<D: EditorViewDelegate> EditorView<D> {
    /// Create with a specific delegate.
    pub fn with_delegate(delegate: D) -> Self {
        Self {
            group: GroupState::new(ViewOptions::default().with_focusable()),
            editor: Editor::from_text(""),
            path: PathBuf::new(),
            highlighter: Highlighter::new(),
            hl_cache: HighlightCache::new(""),
            delegate,
            cmdline_active: false,
            cmdline_prefix: ':',
            match_count: 0,
            tick_count: 0,
        }
    }

    /// Set content and file extension for highlighting.
    pub fn set_content(&mut self, content: &str, ext: &str) {
        self.editor.replace_content(content);
        self.hl_cache = HighlightCache::new(ext);
        self.group.mark_dirty();
    }

    /// Get full buffer content as string.
    pub fn content(&self) -> String {
        self.editor.buf().content()
    }

    /// Whether buffer has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.editor.buf().is_modified()
    }

    pub fn editor(&self) -> &Editor {
        &self.editor
    }

    pub fn editor_mut(&mut self) -> &mut Editor {
        &mut self.editor
    }

    pub fn delegate(&self) -> &D {
        &self.delegate
    }

    pub fn delegate_mut(&mut self) -> &mut D {
        &mut self.delegate
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn set_path(&mut self, path: &Path) {
        self.path = path.to_path_buf();
        let ext = extension_from_path(path).to_string();
        self.hl_cache = HighlightCache::new(&ext);
        self.group.set_title(file_title(path));
    }

    pub fn highlighter(&self) -> &Highlighter {
        &self.highlighter
    }

    pub fn highlighter_mut(&mut self) -> &mut Highlighter {
        &mut self.highlighter
    }

    pub fn hl_cache_mut(&mut self) -> &mut HighlightCache {
        &mut self.hl_cache
    }

    pub fn mark_dirty(&mut self) {
        self.group.mark_dirty();
    }

    pub fn is_focused(&self) -> bool {
        self.group.is_focused()
    }

    pub fn put_command(&self, id: u16, data: Option<Box<dyn std::any::Any + Send>>) {
        self.group.put_command(id, data);
    }

    pub fn put_broadcast(&self, id: u16, data: Option<Box<dyn std::any::Any + Send>>) {
        self.group.put_broadcast(id, data);
    }

    pub fn buffer_mut(&mut self) -> &mut txv_core::prelude::Buffer {
        self.group.buffer_mut()
    }

    fn content_height(&self) -> u16 {
        let h = self.group.bounds().h();
        if self.cmdline_active {
            h.saturating_sub(1)
        } else {
            h
        }
    }
}

impl<D: EditorViewDelegate + 'static> View for EditorView<D> {
    delegate_group_state!(group, override { set_bounds, draw, handle, cursor, title, select, unselect, needs_redraw });

    fn title(&self) -> &str {
        if let Some(t) = self.delegate.title(&self.editor) {
            return t;
        }
        self.group.title()
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        if D::supports_downcast() {
            Some(self)
        } else {
            None
        }
    }

    fn needs_redraw(&self) -> bool {
        self.group.any_dirty() || self.delegate.needs_redraw(&self.editor)
    }

    fn can_close(&self) -> CloseResult {
        if let Some(result) = self.delegate.can_close(&self.editor) {
            return result;
        }
        if self.editor.buf().is_modified() {
            CloseResult::Denied("unsaved changes".to_string())
        } else {
            CloseResult::Ok
        }
    }

    fn select(&mut self) {
        self.group.set_focused(true);
        self.group.mark_dirty();
    }

    fn unselect(&mut self) {
        self.group.set_focused(false);
        self.group.mark_dirty();
    }

    fn set_bounds(&mut self, r: Rect) {
        self.group.set_bounds(r);
        self.editor.set_viewport_height(self.content_height() as usize);
        self.relayout_cmdline();
        self.ensure_cursor_visible();
    }

    fn draw(&mut self) {
        self.draw_impl();
        // Draw cmdline prefix and label on last row
        if self.cmdline_active {
            let b = self.group.bounds();
            let y = b.h().saturating_sub(1);
            let style = palette().style(StyleId::StatusBar);
            self.group.buffer_mut().put(0, y, self.cmdline_prefix, style);
            let label_w = self.cmdline_label_width();
            if label_w > 0 {
                let label = format!(" {} found", self.match_count);
                let x = b.w().saturating_sub(label_w);
                self.group.buffer_mut().print(x, y, &label, style);
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        self.handle_impl(event)
    }

    fn cursor(&self) -> Option<CursorRequest> {
        self.cursor_impl()
    }
}

impl<D: EditorViewDelegate> EditorView<D> {
    fn relayout_cmdline(&mut self) {
        if !self.cmdline_active {
            return;
        }
        let b = self.group.bounds();
        let y = b.h().saturating_sub(1);
        let prefix_w: u16 = 1; // ':' or '/' or '?'
        let label_w = self.cmdline_label_width();
        let input_w = b.w().saturating_sub(prefix_w + label_w);
        self.group.set_child_bounds(0, Rect::new(prefix_w, y, input_w, 1));
    }

    fn cmdline_label_width(&self) -> u16 {
        if self.match_count > 0 && self.editor.mode() == EditorMode::Search {
            // " N found"
            let s = format!(" {} found", self.match_count);
            s.len() as u16
        } else {
            0
        }
    }
}

fn file_title(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("untitled")
        .to_string()
}
