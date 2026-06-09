//! Key handling for InputLine.

use txv_core::message::Message;
use txv_core::prelude::*;

use super::{InputLine, CM_CLIPBOARD_PASTE, CM_COPY_TO_CLIPBOARD, CM_PASTE_REQUEST};
use txv_core::commands::CM_CANCEL;

impl InputLine {
    pub(super) fn handle_event(&mut self, event: &Event) -> HandleResult {
        if let Event::Command { id, data, .. } = event {
            if *id == CM_CLIPBOARD_PASTE {
                return self.handle_paste(data);
            }
            return self.handle_command(*id, data);
        }
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        let shift = key.modifiers().shift();
        self.dispatch_key(key, shift)
    }

    fn handle_command(&mut self, id: CommandId, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        if id == CM_COPY_TO_CLIPBOARD || id == CM_PASTE_REQUEST {
            return HandleResult::Ignored;
        }
        if Some(id) == self.prefill_command {
            if let Some(text) = data.as_ref().and_then(|d| d.downcast_ref::<String>()) {
                self.set_text(text);
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }

    fn handle_paste(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        let Some(text) = data.as_ref().and_then(|d| d.downcast_ref::<String>()) else {
            return HandleResult::Consumed;
        };
        let first_line = text.lines().next().unwrap_or("");
        self.insert_text(first_line);
        self.notify_change();
        let line_count = text.lines().count();
        if line_count > 1 {
            let msg = Message::warn("paste", format!("inserted only 1st of {} lines", line_count));
            self.state.put_command(crate::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        HandleResult::Consumed
    }

    fn dispatch_key(&mut self, key: &txv_core::event::KeyEvent, shift: bool) -> HandleResult {
        match key.code() {
            KeyCode::Char(ch) => self.handle_char_key(ch, key, shift),
            KeyCode::Backspace => self.handle_backspace_key(key),
            KeyCode::Delete => {
                self.handle_delete();
                self.notify_change();
                self.update_completions();
                HandleResult::Consumed
            }
            KeyCode::Left => self.handle_nav_left(shift),
            KeyCode::Right => self.handle_nav_right(shift),
            KeyCode::Home => {
                self.handle_nav(shift, 0);
                HandleResult::Consumed
            }
            KeyCode::End => {
                self.handle_nav(shift, self.char_count());
                HandleResult::Consumed
            }
            KeyCode::Up => {
                self.handle_up_key();
                HandleResult::Consumed
            }
            KeyCode::Down => {
                self.handle_down_key();
                HandleResult::Consumed
            }
            KeyCode::Tab => {
                self.try_complete();
                HandleResult::Consumed
            }
            KeyCode::Enter => self.handle_enter_key(),
            KeyCode::Esc => {
                self.hide_sidekick();
                self.state.put_command(CM_CANCEL, None);
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }

    fn handle_backspace_key(&mut self, key: &txv_core::event::KeyEvent) -> HandleResult {
        if key.modifiers().alt() {
            let killed = self.kill_word_back();
            if !killed.is_empty() {
                self.clipboard_copy(&killed);
            }
        } else {
            self.handle_backspace();
        }
        self.notify_change();
        self.update_completions();
        HandleResult::Consumed
    }

    fn handle_nav_left(&mut self, shift: bool) -> HandleResult {
        let new = self.cursor.saturating_sub(1);
        if new != self.cursor || self.selection.is_some() {
            self.handle_nav(shift, new);
        }
        HandleResult::Consumed
    }

    fn handle_nav_right(&mut self, shift: bool) -> HandleResult {
        let max = self.char_count();
        let new = (self.cursor + 1).min(max);
        if new != self.cursor || self.selection.is_some() {
            self.handle_nav(shift, new);
        }
        HandleResult::Consumed
    }

    fn handle_up_key(&mut self) {
        if self.sidekick_visible {
            self.sidekick_select_prev();
        } else {
            self.handle_history_up();
        }
    }

    fn handle_down_key(&mut self) {
        if self.sidekick_visible {
            self.sidekick_select_next();
        } else {
            self.handle_history_down();
        }
    }

    fn handle_enter_key(&mut self) -> HandleResult {
        if self.sidekick_visible {
            self.apply_sidekick_selection();
        }
        self.hide_sidekick();
        self.push_history();
        self.state
            .put_command(self.submit_command, Some(Box::new(self.text.clone())));
        HandleResult::Consumed
    }

    fn handle_char_key(&mut self, ch: char, key: &txv_core::event::KeyEvent, shift: bool) -> HandleResult {
        if key.modifiers().alt() {
            return self.handle_alt_char(ch, shift);
        }
        if key.modifiers().ctrl() {
            return self.handle_ctrl_char(ch, shift);
        }
        self.handle_char(ch);
        self.update_completions();
        HandleResult::Consumed
    }

    fn handle_alt_char(&mut self, ch: char, shift: bool) -> HandleResult {
        match ch {
            'f' => self.handle_nav(shift, self.word_forward()),
            'b' => self.handle_nav(shift, self.word_backward()),
            'd' => {
                let killed = self.kill_word_forward();
                if !killed.is_empty() {
                    self.clipboard_copy(&killed);
                }
                self.notify_change();
                self.update_completions();
            }
            _ => return HandleResult::Ignored,
        }
        HandleResult::Consumed
    }

    fn handle_ctrl_char(&mut self, ch: char, shift: bool) -> HandleResult {
        match ch {
            'a' => self.handle_nav(shift, 0),
            'e' => self.handle_nav(shift, self.char_count()),
            'f' => self.handle_nav(shift, (self.cursor + 1).min(self.char_count())),
            'b' => self.handle_nav(shift, self.cursor.saturating_sub(1)),
            'd' => {
                self.handle_delete();
                self.notify_change();
                self.update_completions();
            }
            'k' => self.kill_and_copy(Self::kill_to_end),
            'u' => self.kill_and_copy(Self::kill_to_start),
            'w' => self.kill_and_copy(Self::kill_word_back),
            't' => self.transpose_chars(),
            'y' | 'v' => return self.handle_ctrl_paste(),
            'c' => return self.handle_ctrl_copy(),
            _ => return HandleResult::Ignored,
        }
        HandleResult::Consumed
    }

    fn kill_and_copy(&mut self, op: fn(&mut Self) -> String) {
        let killed = op(self);
        if !killed.is_empty() {
            self.clipboard_copy(&killed);
        }
        self.notify_change();
        self.update_completions();
    }

    fn handle_ctrl_paste(&mut self) -> HandleResult {
        let text = self
            .clipboard
            .as_ref()
            .and_then(|c| c.lock().ok())
            .and_then(|mut r| r.paste());
        if let Some(text) = text {
            let first_line = text.lines().next().unwrap_or("").to_string();
            self.insert_text(&first_line);
            self.notify_change();
        } else {
            self.state.put_command(CM_PASTE_REQUEST, None);
        }
        HandleResult::Consumed
    }

    fn handle_ctrl_copy(&mut self) -> HandleResult {
        if let Some(text) = self.selected_text() {
            self.clipboard_copy(&text);
        }
        HandleResult::Consumed
    }
}
