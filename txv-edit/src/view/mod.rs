//! EditorView — a reusable View wrapping txv_edit::editor::Editor.

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

/// A reusable editor View. Parameterized by a delegate for app extensions.
pub struct EditorView<D: EditorViewDelegate = NullDelegate> {
    state: ViewState,
    editor: Editor,
    path: PathBuf,
    highlighter: Highlighter,
    hl_cache: HighlightCache,
    delegate: D,
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
            state: ViewState::default(),
            editor,
            path: path.to_path_buf(),
            highlighter: Highlighter::new(),
            hl_cache: HighlightCache::new(&ext),
            delegate: NullDelegate,
        };
        view.state.set_title(file_title(path));
        Ok(view)
    }
}

impl<D: EditorViewDelegate> EditorView<D> {
    /// Create with a specific delegate.
    pub fn with_delegate(delegate: D) -> Self {
        Self {
            state: ViewState::default(),
            editor: Editor::from_text(""),
            path: PathBuf::new(),
            highlighter: Highlighter::new(),
            hl_cache: HighlightCache::new(""),
            delegate,
        }
    }

    /// Set content and file extension for highlighting.
    pub fn set_content(&mut self, content: &str, ext: &str) {
        self.editor.replace_content(content);
        self.hl_cache = HighlightCache::new(ext);
        self.state.mark_dirty();
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
        self.state.set_title(file_title(path));
    }

    pub fn state(&self) -> &ViewState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut ViewState {
        &mut self.state
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
}

impl<D: EditorViewDelegate + 'static> View for EditorView<D> {
    delegate_view_state!(state, override { set_bounds, cursor, title });

    fn title(&self) -> &str {
        self.state.title()
    }

    fn set_bounds(&mut self, r: Rect) {
        self.state.set_bounds(r);
        let h = r.h() as usize;
        self.editor.set_viewport_height(h);
    }

    fn draw(&mut self) {
        self.draw_impl();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        self.handle_impl(event)
    }

    fn cursor(&self) -> Option<CursorRequest> {
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
        let cursor_style = match self.editor.mode() {
            EditorMode::Insert => opts.cursor_insert(),
            EditorMode::Command | EditorMode::Search => opts.cursor_command(),
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

fn file_title(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("untitled")
        .to_string()
}
