//! EditorView parts: the editor, the app delegate, and syntax highlighting.

use super::{EditorView, EditorViewDelegate};
use crate::editor::Editor;
use crate::highlight::{HighlightCache, Highlighter};

impl<D: EditorViewDelegate> EditorView<D> {
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

    /// Delegate and editor together, for delegate methods that act on the
    /// editor outside the event path (the delegate's hooks get both).
    pub fn delegate_and_editor_mut(&mut self) -> (&mut D, &mut Editor) {
        (&mut self.delegate, &mut self.editor)
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
