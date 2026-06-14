//! handle() implementation for EditorView.

use txv_core::prelude::*;
use txv_core::text::display_width;

use super::{EditorView, CM_EDITOR_CLOSE, CM_EDITOR_CONTENT_CHANGED, CM_EDITOR_CURSOR_MOVED, CM_EDITOR_SAVE};
use crate::editor::command::Command;
use crate::editor::keymap::Keymap;
use crate::editor::motions::word_at;
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
        if *cmd == Command::LspRename {
            let word =
                word_at(&self.editor.buf(), self.editor.cursor_line(), self.editor.cursor_col()).unwrap_or_default();
            self.activate_cmdline_with_text(":", &format!("lsp-rename {word}"));
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

    pub fn ensure_cursor_visible(&mut self) {
        let line = self.editor.cursor_line();
        let scroll = self.editor.viewport_scroll();
        let height = self.editor.viewport_height();
        let scrolloff = self.editor.options().scrolloff();
        if height == 0 {
            return;
        }
        if self.editor.options().wrap() {
            self.ensure_cursor_visible_wrapped(line, scroll, height, scrolloff);
        } else {
            self.ensure_cursor_visible_nowrap(line, scroll, height, scrolloff);
        }
    }

    fn ensure_cursor_visible_nowrap(&mut self, line: usize, scroll: usize, height: usize, scrolloff: usize) {
        if line < scroll + scrolloff {
            self.editor.set_viewport_scroll(line.saturating_sub(scrolloff));
        } else if line >= scroll + height - scrolloff {
            self.editor
                .set_viewport_scroll(line.saturating_sub(height - 1 - scrolloff));
        }
        // Horizontal scroll
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

    fn ensure_cursor_visible_wrapped(&mut self, line: usize, scroll: usize, height: usize, scrolloff: usize) {
        let avail = self.text_avail();
        if avail == 0 {
            return;
        }
        // Compute visual row of cursor relative to scroll position
        let cursor_vrow = self.visual_rows_between(scroll, line, avail);
        let cursor_line_h = self.wrapped_line_height(line, avail);
        // Scroll up: cursor is above visible area
        if line < scroll + scrolloff {
            self.editor.set_viewport_scroll(line.saturating_sub(scrolloff));
            return;
        }
        // Scroll down: cursor's last visual row is below viewport
        if cursor_vrow + cursor_line_h > height.saturating_sub(scrolloff) {
            // Find new scroll so cursor fits within viewport with scrolloff margin
            let target_vrow = height.saturating_sub(scrolloff).saturating_sub(cursor_line_h);
            let new_scroll = self.find_scroll_for_visual_row(line, target_vrow, avail);
            self.editor.set_viewport_scroll(new_scroll);
        }
    }

    /// Available text columns (viewport width minus gutter).
    fn text_avail(&self) -> usize {
        let gw = self.gutter_width() as usize;
        (self.group.bounds().w() as usize).saturating_sub(gw)
    }

    /// How many visual rows a single buffer line occupies with wrapping.
    fn wrapped_line_height(&self, line_idx: usize, avail: usize) -> usize {
        let line = self.editor.buf().line(line_idx).unwrap_or_default();
        let tw = self.editor.options().tab_width();
        let w = display_width(&line, tw) as usize;
        if w == 0 {
            1
        } else {
            w.div_ceil(avail)
        }
    }

    /// Sum of visual rows for buffer lines [from..to) with wrapping.
    fn visual_rows_between(&self, from: usize, to: usize, avail: usize) -> usize {
        let mut rows = 0;
        for i in from..to {
            rows += self.wrapped_line_height(i, avail);
        }
        rows
    }

    /// Find the scroll (buffer line) such that `target_line` appears at approximately
    /// `target_vrow` visual rows from the top.
    fn find_scroll_for_visual_row(&self, target_line: usize, target_vrow: usize, avail: usize) -> usize {
        // Walk backwards from target_line, accumulating visual rows
        let mut vrows = 0;
        let mut s = target_line;
        while s > 0 {
            s -= 1;
            let h = self.wrapped_line_height(s, avail);
            if vrows + h > target_vrow {
                return s + 1;
            }
            vrows += h;
        }
        0
    }
}
