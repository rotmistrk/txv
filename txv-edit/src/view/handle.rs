//! handle() implementation for EditorView.

use txv_core::prelude::*;

use super::{EditorView, CM_EDITOR_CLOSE, CM_EDITOR_CONTENT_CHANGED, CM_EDITOR_CURSOR_MOVED, CM_EDITOR_SAVE};
use crate::editor::command::Command;
use crate::editor::keymap::Keymap;
use crate::editor::EditorAction;
use crate::view::delegate::EditorViewDelegate;

impl<D: EditorViewDelegate> EditorView<D> {
    pub(super) fn handle_impl(&mut self, event: &Event) -> HandleResult {
        match event {
            Event::Key(key) => self.handle_key(*key),
            Event::Tick => HandleResult::Ignored,
            _ => HandleResult::Ignored,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        let mode = self.editor.mode();
        let cmd = self.editor.keymap_mut().handle_key(&key, mode);

        if cmd == Command::Noop {
            return HandleResult::Consumed;
        }

        let action = self.editor.execute(cmd);
        self.process_action(&action);
        self.ensure_cursor_visible();
        self.state.mark_dirty();
        HandleResult::Consumed
    }

    fn process_action(&mut self, action: &EditorAction) {
        if self.delegate.on_action(action) {
            return;
        }
        match action {
            EditorAction::SaveRequested => {
                self.state.put_command(CM_EDITOR_SAVE, None);
            }
            EditorAction::CloseRequested | EditorAction::ForceCloseRequested => {
                self.state.put_command(CM_EDITOR_CLOSE, None);
            }
            EditorAction::CursorMoved => {
                self.state.put_command(CM_EDITOR_CURSOR_MOVED, None);
            }
            EditorAction::ContentChanged => {
                self.hl_cache.invalidate_from(self.editor.cursor_line());
                self.state.put_command(CM_EDITOR_CONTENT_CHANGED, None);
            }
            _ => {}
        }
    }

    fn ensure_cursor_visible(&mut self) {
        let line = self.editor.cursor_line();
        let scroll = self.editor.viewport_scroll();
        let height = self.editor.viewport_height();
        let scrolloff = self.editor.options().scrolloff();

        if height == 0 {
            return;
        }

        if line < scroll + scrolloff {
            self.editor.set_viewport_scroll(line.saturating_sub(scrolloff));
        } else if line >= scroll + height - scrolloff {
            self.editor
                .set_viewport_scroll(line.saturating_sub(height - 1 - scrolloff));
        }
    }
}
