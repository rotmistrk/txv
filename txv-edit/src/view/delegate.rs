//! EditorViewDelegate — trait for app-specific extensions.

use txv_core::prelude::{Buffer, Style};

use crate::editor::{Editor, EditorAction};

/// Delegate trait for app-specific extensions. All methods have default no-ops.
pub trait EditorViewDelegate: Send {
    /// Extra style for a character at (line, col). Used for diagnostic underlines, etc.
    fn extra_style(&self, _line: usize, _col: usize) -> Option<Style> {
        None
    }

    /// Draw extra content in the gutter (blame annotations, git signs, etc.)
    fn draw_gutter_sign(&self, _buf: &mut Buffer, _line: usize, _x: u16, _y: u16) {}

    /// Called after the main draw pass. Use for overlays (diagnostics, popups).
    fn post_draw(&self, _buf: &mut Buffer, _editor: &Editor) {}

    /// Called when the editor produces an action. Return true if handled.
    fn on_action(&mut self, _action: &EditorAction) -> bool {
        false
    }

    /// Gutter width addition (extra columns beyond line numbers).
    fn extra_gutter_width(&self) -> u16 {
        0
    }
}

/// No-op delegate for standalone usage.
pub struct NullDelegate;

impl EditorViewDelegate for NullDelegate {}
