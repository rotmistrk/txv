//! EditorViewDelegate — trait for app-specific extensions.

use std::any::Any;

use txv_core::event::{CommandId, KeyEvent};
use txv_core::prelude::{palette, Color, Style, StyleId};
use txv_core::view::HandleResult;

use crate::editor::keymap::EditorMode;
use crate::editor::{Editor, EditorAction};

pub use super::highlight_range::{CursorRender, HighlightRange};
pub use super::line_decoration::{DecorationStyle, LineDecoration};

/// Delegate trait for app-specific extensions. All methods have default no-ops.
pub trait EditorViewDelegate: Send {
    /// Extra style for a character at (line, col). For semantic token coloring.
    fn extra_style(&self, _line: usize, _col: usize) -> Option<Style> {
        None
    }

    /// Left gutter sign for a line (e.g. git markers). Drawn after line numbers.
    fn gutter_sign(&self, _line: usize) -> Option<(char, Style)> {
        None
    }

    /// Right gutter sign for a line (e.g. diagnostics). Drawn adjacent to text.
    fn gutter_sign_right(&self, _line: usize) -> Option<(char, Style)> {
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

    fn cmdline_completer(&self) -> Option<Box<dyn txv_core::complete::Completer>> {
        None
    }

    /// Shared history for : commands. Application provides for cross-editor sharing.
    fn command_history(&self) -> Option<txv_core::shared_history::SharedHistory> {
        None
    }

    /// Shared history for / and ? search. Application provides for cross-editor sharing.
    fn search_history(&self) -> Option<txv_core::shared_history::SharedHistory> {
        None
    }

    fn drain_commands(&mut self) -> Vec<(u16, Option<Box<dyn Any + Send>>)> {
        vec![]
    }
    fn drain_broadcasts(&mut self) -> Vec<(u16, Option<Box<dyn Any + Send>>)> {
        vec![]
    }
}

/// No-op delegate for standalone usage.
pub struct NullDelegate;

impl EditorViewDelegate for NullDelegate {}
