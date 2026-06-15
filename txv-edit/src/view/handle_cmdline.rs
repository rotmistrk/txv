//! Command-line (: / ?) handling for EditorView.

use txv_core::prelude::*;

use super::{EditorView, CM_CMDLINE_CHANGED};
use crate::editor::command::Command;
use crate::editor::keymap::EditorMode;
use crate::editor::EditorAction;
use crate::view::delegate::EditorViewDelegate;

impl<D: EditorViewDelegate> EditorView<D> {
    pub(super) fn handle_cmdline_event(&mut self, event: &Event) -> HandleResult {
        if let Event::Command { id, data, .. } = event {
            return self.handle_cmdline_command(*id, data);
        }
        let result = self.group.dispatch(event);
        // Drain commands emitted by InputLine child (e.g. CM_OK on Enter)
        if let Some(r) = self.drain_child_commands() {
            return r;
        }
        self.group.mark_dirty();
        result
    }

    fn drain_child_commands(&mut self) -> Option<HandleResult> {
        let sink = self.group.sink()?;
        let events = sink.drain();
        for ev in events {
            if let Event::Command { id, data, .. } = ev {
                let r = self.handle_cmdline_command(id, &data);
                if r != HandleResult::Ignored {
                    return Some(r);
                }
                // Re-emit unhandled commands so they reach outer components
                self.group.put_command(id, data);
            }
        }
        None
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

    pub(super) fn activate_cmdline(&mut self, prefix: &str) {
        use txv_widgets::InputLine;
        let is_search = prefix == "/" || prefix == "?";
        let mode = if is_search {
            EditorMode::Search
        } else {
            EditorMode::Command
        };
        let old_mode = self.editor.mode();
        self.editor.set_mode(mode);
        self.delegate.on_mode_changed(old_mode, mode, &self.editor);
        self.cmdline_prefix = prefix.chars().next().unwrap_or(':');
        self.match_count = 0;

        let mut il = InputLine::new();
        if is_search {
            il = il.with_change_command(CM_CMDLINE_CHANGED);
            if let Some(h) = self.delegate.search_history() {
                il = il.with_history(h);
            }
        } else {
            if let Some(c) = self.delegate.cmdline_completer() {
                il = il.with_completer(c);
            }
            if let Some(h) = self.delegate.command_history() {
                il = il.with_history(h);
            }
        }

        self.group.insert(Box::new(il));
        self.group.set_focused_index(0);
        self.group.select_focused();
        self.cmdline_active = true;
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

    pub(super) fn activate_cmdline_with_text(&mut self, prefix: &str, text: &str) {
        self.activate_cmdline(prefix);
        if let Some(il) = self.group.child_mut(0).and_then(|v| v.as_any_mut()) {
            if let Some(il) = il.downcast_mut::<txv_widgets::InputLine>() {
                il.set_text(text);
            }
        }
    }

    fn cmdline_submit(&mut self, text: String) {
        let mode = self.editor.mode();
        self.deactivate_cmdline();
        self.editor.set_mode(EditorMode::Normal);
        self.delegate.on_mode_changed(mode, EditorMode::Normal, &self.editor);
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
        self.delegate.on_action_post(&action, &self.editor);
        if matches!(action, EditorAction::CursorMoved | EditorAction::ContentChanged) {
            self.delegate.on_cursor_moved(&self.editor);
        }
    }

    fn cmdline_cancel(&mut self) {
        if let Some((line, col)) = self.editor.incsearch_origin.take() {
            self.editor.set_cursor_line(line);
            self.editor.set_cursor_col(col);
        }
        let old_mode = self.editor.mode();
        self.deactivate_cmdline();
        self.editor.set_mode(EditorMode::Normal);
        self.delegate
            .on_mode_changed(old_mode, EditorMode::Normal, &self.editor);
    }

    pub(super) fn incsearch_update(&mut self) {
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
}
