//! handle() implementation for EditorView.

use txv_core::prelude::*;

use super::{EditorView, CM_EDITOR_CLOSE, CM_EDITOR_CONTENT_CHANGED, CM_EDITOR_CURSOR_MOVED, CM_EDITOR_SAVE};
use crate::editor::command::Command;
use crate::editor::keymap::Keymap;
use crate::editor::EditorAction;
use crate::view::delegate::EditorViewDelegate;

impl<D: EditorViewDelegate> EditorView<D> {
    pub(super) fn handle_impl(&mut self, event: &Event) -> HandleResult {
        if self.cmdline_active {
            return self.handle_cmdline_event(event);
        }
        match event {
            Event::Tick => {
                self.tick_count += 1;
                self.delegate.on_tick(&mut self.editor, self.tick_count)
            }
            Event::Command { id, data, .. } => self.delegate.on_command(*id, data, &mut self.editor),
            Event::Paste(text) => self.delegate.on_paste(text, &mut self.editor),
            Event::Key(key) => self.handle_key(*key),
            _ => HandleResult::Ignored,
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> HandleResult {
        if let Some(result) = self.delegate.on_key_pre(&key, &mut self.editor) {
            return result;
        }
        let old_mode = self.editor.mode();
        let old_line = self.editor.cursor_line();
        let old_col = self.editor.cursor_col();

        let cmd = self.editor.keymap_mut().handle_key(&key, old_mode);
        if cmd == Command::Noop {
            return HandleResult::Consumed;
        }
        if let Some(r) = self.check_mode_entry(&cmd) {
            return r;
        }

        let is_search = Self::is_search_command(&cmd);
        let action = self.editor.execute(cmd);
        self.process_action(&action);
        self.ensure_cursor_visible();
        if !is_search || !self.editor.options().hlsearch() {
            self.editor.set_highlight(None);
        }

        self.delegate.on_action_post(&action, &self.editor);
        let new_mode = self.editor.mode();
        if new_mode != old_mode {
            self.delegate.on_mode_changed(old_mode, new_mode, &self.editor);
        }
        if self.editor.cursor_line() != old_line || self.editor.cursor_col() != old_col {
            self.delegate.on_cursor_moved(&self.editor);
        }

        self.group.mark_dirty();
        HandleResult::Consumed
    }

    fn is_search_command(cmd: &Command) -> bool {
        matches!(
            cmd,
            Command::SearchNext
                | Command::SearchPrev
                | Command::SearchWordForward
                | Command::SearchWordBackward
                | Command::SearchForward(_)
                | Command::SearchBackward(_)
        )
    }

    fn check_mode_entry(&mut self, cmd: &Command) -> Option<HandleResult> {
        if *cmd == Command::EnterCommandMode {
            self.activate_cmdline(":");
            return Some(HandleResult::Consumed);
        }
        if *cmd == Command::VisualExCommand {
            self.editor.save_visual_range();
            self.activate_cmdline_with_text(":", "'<,'>");
            return Some(HandleResult::Consumed);
        }
        if *cmd == Command::EnterSearchMode {
            self.editor.incsearch_origin = Some((self.editor.cursor_line(), self.editor.cursor_col()));
            self.editor.search_direction_forward = true;
            self.activate_cmdline("/");
            return Some(HandleResult::Consumed);
        }
        if *cmd == Command::EnterSearchBackward {
            self.editor.incsearch_origin = Some((self.editor.cursor_line(), self.editor.cursor_col()));
            self.editor.search_direction_forward = false;
            self.activate_cmdline("?");
            return Some(HandleResult::Consumed);
        }
        None
    }

    pub(super) fn process_action(&mut self, action: &EditorAction) {
        if self.delegate.on_action(action) {
            return;
        }
        match action {
            EditorAction::SaveRequested => {
                self.group.put_command(CM_EDITOR_SAVE, None);
            }
            EditorAction::CloseRequested | EditorAction::ForceCloseRequested => {
                self.group.put_command(CM_EDITOR_CLOSE, None);
            }
            EditorAction::CursorMoved => {
                self.group.put_command(CM_EDITOR_CURSOR_MOVED, None);
            }
            EditorAction::ContentChanged => {
                self.hl_cache.invalidate_from(self.editor.cursor_line());
                self.group.put_command(CM_EDITOR_CONTENT_CHANGED, None);
            }
            _ => {}
        }
    }

    pub(super) fn ensure_cursor_visible(&mut self) {
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
        // Horizontal scroll
        if !self.editor.options().wrap() {
            let col = self.editor.cursor_col();
            let gw = self.gutter_width() as usize;
            let avail = (self.group.bounds().w() as usize).saturating_sub(gw);
            let h_off = self.editor.h_scroll();
            if col < h_off {
                self.editor.set_h_scroll(col);
            } else if avail > 0 && col >= h_off + avail {
                self.editor.set_h_scroll(col - avail + 1);
            }
        }
    }
}
