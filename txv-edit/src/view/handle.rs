//! handle() implementation for EditorView.

use txv_core::prelude::*;

use super::{
    EditorView, CM_CMDLINE_CHANGED, CM_EDITOR_CLOSE, CM_EDITOR_CONTENT_CHANGED, CM_EDITOR_CURSOR_MOVED, CM_EDITOR_SAVE,
};
use crate::editor::command::Command;
use crate::editor::keymap::{EditorMode, Keymap};
use crate::editor::EditorAction;
use crate::view::delegate::EditorViewDelegate;

impl<D: EditorViewDelegate> EditorView<D> {
    pub(super) fn handle_impl(&mut self, event: &Event) -> HandleResult {
        // If cmdline active, dispatch to InputLine child first
        if self.cmdline_active {
            return self.handle_cmdline_event(event);
        }
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

        // Intercept command/search mode entry — spawn InputLine
        if cmd == Command::EnterCommandMode {
            self.activate_cmdline(":");
            return HandleResult::Consumed;
        }
        if cmd == Command::EnterSearchMode {
            self.editor.incsearch_origin = Some((self.editor.cursor_line(), self.editor.cursor_col()));
            self.editor.search_direction_forward = true;
            self.activate_cmdline("/");
            return HandleResult::Consumed;
        }
        if cmd == Command::EnterSearchBackward {
            self.editor.incsearch_origin = Some((self.editor.cursor_line(), self.editor.cursor_col()));
            self.editor.search_direction_forward = false;
            self.activate_cmdline("?");
            return HandleResult::Consumed;
        }

        let is_search = matches!(
            cmd,
            Command::SearchNext
                | Command::SearchPrev
                | Command::SearchWordForward
                | Command::SearchWordBackward
                | Command::SearchForward(_)
                | Command::SearchBackward(_)
        );

        let action = self.editor.execute(cmd);
        self.process_action(&action);
        self.ensure_cursor_visible();
        if !is_search || !self.editor.options().hlsearch() {
            self.editor.set_highlight(None);
        }
        self.group.mark_dirty();
        HandleResult::Consumed
    }

    fn handle_cmdline_event(&mut self, event: &Event) -> HandleResult {
        // Check for submit/cancel/change commands from InputLine
        if let Event::Command { id, data, .. } = event {
            return self.handle_cmdline_command(*id, data);
        }

        // Dispatch key to InputLine child
        let result = self.group.dispatch(event);
        self.group.mark_dirty();
        result
    }

    fn handle_cmdline_command(&mut self, id: CommandId, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        match id {
            CM_OK => {
                let text = data
                    .as_ref()
                    .and_then(|d| d.downcast_ref::<String>())
                    .cloned()
                    .unwrap_or_default();
                self.cmdline_submit(text);
                HandleResult::Consumed
            }
            CM_CANCEL => {
                self.cmdline_cancel();
                HandleResult::Consumed
            }
            CM_CMDLINE_CHANGED => {
                self.incsearch_update();
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }

    fn activate_cmdline(&mut self, prefix: &str) {
        use txv_widgets::InputLine;
        let is_search = prefix == "/" || prefix == "?";
        let mode = if is_search {
            EditorMode::Search
        } else {
            EditorMode::Command
        };
        self.editor.set_mode(mode);
        self.cmdline_prefix = prefix.chars().next().unwrap_or(':');
        self.match_count = 0;

        let mut il = InputLine::new();
        if is_search {
            il = il.with_change_command(CM_CMDLINE_CHANGED);
        }

        self.group.insert(Box::new(il));
        self.group.set_focused_index(0);
        self.group.select_focused();
        self.cmdline_active = true;

        // Shrink editor viewport
        self.editor.set_viewport_height(self.content_height() as usize);
        self.relayout_cmdline();
        self.group.mark_dirty();
    }

    fn deactivate_cmdline(&mut self) {
        if self.cmdline_active {
            self.group.remove(0);
            self.cmdline_active = false;
            self.editor.set_viewport_height(self.content_height() as usize);
            self.group.mark_dirty();
        }
    }

    fn cmdline_submit(&mut self, text: String) {
        let mode = self.editor.mode();
        self.deactivate_cmdline();
        self.editor.set_mode(EditorMode::Normal);
        self.editor.incsearch_origin = None;

        if text.is_empty() {
            return;
        }

        let action = if mode == EditorMode::Search {
            self.editor.search_pattern = text;
            self.editor.update_highlight();
            EditorAction::CursorMoved
        } else {
            self.editor.execute(Command::ExCommand(text))
        };
        self.process_action(&action);
        self.ensure_cursor_visible();
    }

    fn cmdline_cancel(&mut self) {
        // Restore cursor if incremental search was active
        if let Some((line, col)) = self.editor.incsearch_origin.take() {
            self.editor.set_cursor_line(line);
            self.editor.set_cursor_col(col);
        }
        self.deactivate_cmdline();
        self.editor.set_mode(EditorMode::Normal);
    }

    fn incsearch_update(&mut self) {
        // Read current text from InputLine child
        let text = self
            .group
            .child_mut(0)
            .and_then(|v| v.as_any_mut())
            .and_then(|a| a.downcast_ref::<txv_widgets::InputLine>())
            .map(|il| il.text().to_string())
            .unwrap_or_default();

        if text.is_empty() {
            self.match_count = 0;
            if let Some((line, col)) = self.editor.incsearch_origin {
                self.editor.set_cursor_line(line);
                self.editor.set_cursor_col(col);
            }
            self.editor.update_highlight();
            self.relayout_cmdline();
            return;
        }

        let content = self.editor.buf().content();
        // Count matches
        self.match_count = content.matches(&text).count();
        self.relayout_cmdline();

        let origin = self.editor.incsearch_origin.unwrap_or((0, 0));
        let start = self.editor.buf().line_col_to_offset(origin.0, origin.1).unwrap_or(0);
        if self.editor.search_direction_forward {
            if let Some(pos) = content[start..].find(&text) {
                let (l, c) = self.editor.buf().offset_to_line_col(start + pos);
                self.editor.set_cursor_line(l);
                self.editor.set_cursor_col(c);
            }
        } else if let Some(pos) = content[..start].rfind(&text) {
            let (l, c) = self.editor.buf().offset_to_line_col(pos);
            self.editor.set_cursor_line(l);
            self.editor.set_cursor_col(c);
        }
        self.editor.search_pattern = text;
        self.editor.update_highlight();
        self.ensure_cursor_visible();
        self.group.mark_dirty();
    }

    fn process_action(&mut self, action: &EditorAction) {
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
