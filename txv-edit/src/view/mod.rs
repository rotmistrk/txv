//! EditorView — a reusable View wrapping txv_edit::editor::Editor.
//!
//! Uses GroupState to host an InputLine for command/search mode.
//! Child 0 (when present): InputLine for `:` or `/` input.

pub mod delegate;
pub mod draw;
mod handle;

use std::path::{Path, PathBuf};

use txv_core::prelude::*;

pub use delegate::{EditorViewDelegate, NullDelegate};

use crate::editor::keymap::EditorMode;
use crate::editor::Editor;
use crate::highlight::{extension_from_path, HighlightCache, Highlighter};
use crate::settings::CursorStyle;
use crate::view::draw::sticky::sticky_line_count;

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
    delegate_group_state!(group, override { set_bounds, draw, handle, cursor, title, select, unselect });

    fn title(&self) -> &str {
        self.group.title()
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
        let mode = self.editor.mode();

        // Command/Search: delegate to InputLine child cursor
        if self.cmdline_active {
            if let Some(child) = self.group.focused_child() {
                if let Some(req) = child.cursor() {
                    let (ox, oy) = self.group.child_origin(self.group.focused_index());
                    let x = req.x().saturating_add(ox);
                    let y = req.y().saturating_add(oy);
                    return Some(CursorRequest::new(x, y, req.shape()));
                }
            }
            // Fallback: bar cursor at end of prompt
            let h = self.group.bounds().h();
            let y = h.saturating_sub(1);
            return Some(CursorRequest::new(1, y, CursorShape::Bar));
        }

        let gw = self.gutter_width();
        let line = self.editor.cursor_line();
        let col = self.editor.cursor_col();
        let scroll = self.editor.viewport_scroll();
        let h_scroll = self.editor.h_scroll();

        if line < scroll {
            return None;
        }
        let sticky_h = sticky_line_count(&self.editor);
        let y = (line - scroll) as u16 + sticky_h;
        let x = gw + (col.saturating_sub(h_scroll)) as u16;

        let opts = self.editor.options();
        let cursor_style = match mode {
            EditorMode::Insert => opts.cursor_insert(),
            _ => opts.cursor_normal(),
        };
        if cursor_style == CursorStyle::Software {
            return None;
        }
        let shape = match cursor_style {
            CursorStyle::Bar => CursorShape::Bar,
            CursorStyle::Block => CursorShape::Block,
            CursorStyle::Underline => CursorShape::Underline,
            CursorStyle::Software => return None,
        };
        Some(CursorRequest::new(x, y, shape))
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
