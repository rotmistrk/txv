//! EditorViewDelegate — trait for app-specific extensions.

use std::any::Any;

use txv_core::event::{CommandId, KeyEvent};
use txv_core::prelude::{palette, Color, Style, StyleId};
use txv_core::view::HandleResult;

use crate::editor::keymap::EditorMode;
use crate::editor::{Editor, EditorAction};

/// A decoration on a line segment (underline, squiggly, background).
pub struct LineDecoration {
    pub col_start: usize,
    pub col_end: usize,
    pub style: DecorationStyle,
}

/// Visual style for a line decoration.
pub enum DecorationStyle {
    Underline(Color),
    Squiggly(Color),
    Background(Color),
}

/// A highlighted range on a line — merged onto existing cell style.
pub struct HighlightRange {
    pub col_start: usize,
    pub col_end: usize,
    pub style: Style,
}

/// Cursor rendering mode.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CursorRender {
    Hardware,
    Software(Style),
    None,
}

/// Delegate trait for app-specific extensions. All methods have default no-ops.
pub trait EditorViewDelegate: Send {
    /// Extra style for a character at (line, col). For semantic token coloring.
    fn extra_style(&self, _line: usize, _col: usize) -> Option<Style> {
        None
    }

    /// Gutter sign for a line. Returns (char, style) for the sign column.
    fn gutter_sign(&self, _line: usize) -> Option<(char, Style)> {
        None
    }

    /// Line decorations (underlines, squiggly, background ranges).
    fn line_decorations(&self, _line: usize) -> &[LineDecoration] {
        &[]
    }

    /// Highlighted ranges on a line (word highlight, etc.). Merged with normal style.
    fn highlight_ranges(&self, _line: usize) -> &[HighlightRange] {
        &[]
    }

    /// How to render the cursor. Default: hardware terminal cursor.
    fn cursor_render(&self, _mode: EditorMode) -> CursorRender {
        CursorRender::Hardware
    }

    /// Called when the editor produces an action. Return true if handled.
    fn on_action(&mut self, _action: &EditorAction) -> bool {
        false
    }

    /// Gutter width addition (extra columns beyond line numbers).
    fn extra_gutter_width(&self) -> u16 {
        0
    }

    /// Style for current search match highlight.
    fn highlight_match_style(&self) -> Style {
        palette().style(StyleId::SearchMatch)
    }

    /// Background color for non-current search matches and ephemeral highlights.
    fn highlight_other_bg(&self) -> Color {
        palette().style(StyleId::CursorUnfocused).bg()
    }

    /// Style for matching parenthesis/bracket highlight.
    fn matchparen_style(&self) -> Style {
        palette().style(StyleId::SearchMatch)
    }

    // --- Event hooks ---

    fn on_tick(&mut self, _editor: &mut Editor, _tick: u64) -> HandleResult {
        HandleResult::Ignored
    }

    fn on_command(
        &mut self,
        _id: CommandId,
        _data: &Option<Box<dyn Any + Send>>,
        _editor: &mut Editor,
    ) -> HandleResult {
        HandleResult::Ignored
    }

    fn on_paste(&mut self, _text: &str, _editor: &mut Editor) -> HandleResult {
        HandleResult::Ignored
    }

    fn on_key_pre(&mut self, _key: &KeyEvent, _editor: &mut Editor) -> Option<HandleResult> {
        None
    }

    // --- Lifecycle hooks ---

    fn on_action_post(&mut self, _action: &EditorAction, _editor: &Editor) {}
    fn on_cursor_moved(&mut self, _editor: &Editor) {}
    fn on_mode_changed(&mut self, _old: EditorMode, _new: EditorMode, _editor: &Editor) {}

    // --- View trait overrides ---

    fn title(&self, _editor: &Editor) -> Option<&str> {
        None
    }

    fn can_close(&self, _editor: &Editor) -> Option<txv_core::view::CloseResult> {
        None
    }

    fn needs_redraw(&self, _editor: &Editor) -> bool {
        false
    }

    // --- Downcast support ---

    fn supports_downcast() -> bool
    where
        Self: Sized,
    {
        false
    }
}

/// No-op delegate for standalone usage.
pub struct NullDelegate;

impl EditorViewDelegate for NullDelegate {}
