//! Key handling for InputLine.

use txv_core::prelude::*;

use super::{InputLine, CM_CLIPBOARD_PASTE, CM_COPY_TO_CLIPBOARD, CM_PASTE_REQUEST};
use txv_core::commands::CM_CANCEL;

impl InputLine {
    pub(super) fn handle_event(&mut self, event: &Event) -> HandleResult {
        if let Event::Command { id, data, .. } = event {
            if *id == CM_CLIPBOARD_PASTE {
                if let Some(text) = data.as_ref().and_then(|d| d.downcast_ref::<String>()) {
                    let first_line = text.lines().next().unwrap_or("");
                    self.insert_text(first_line);
                    let line_count = text.lines().count();
                    if line_count > 1 {
                        let msg = txv_core::message::Message::warn(
                            "paste",
                            format!("inserted only 1st of {} lines", line_count),
                        );
                        self.state.put_command(crate::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                    }
                    return HandleResult::Consumed;
                }
            }
            return self.handle_command(data);
        }
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        let shift = key.modifiers.shift;
        match &key.code {
            KeyCode::Char(ch) => self.handle_char_key(*ch, key, shift),
            KeyCode::Backspace => {
                if key.modifiers.alt {
                    let killed = self.kill_word_back();
                    if !killed.is_empty() {
                        self.state.put_command(CM_COPY_TO_CLIPBOARD, Some(Box::new(killed)));
                    }
                } else {
                    self.handle_backspace();
                }
                self.update_completions();
                HandleResult::Consumed
            }
            KeyCode::Delete => {
                self.handle_delete();
                self.update_completions();
                HandleResult::Consumed
            }
            KeyCode::Left => {
                let new = self.cursor.saturating_sub(1);
                if new != self.cursor || self.selection.is_some() {
                    self.handle_nav(shift, new);
                }
                HandleResult::Consumed
            }
            KeyCode::Right => {
                let max = self.char_count();
                let new = (self.cursor + 1).min(max);
                if new != self.cursor || self.selection.is_some() {
                    self.handle_nav(shift, new);
                }
                HandleResult::Consumed
            }
            KeyCode::Home => {
                self.handle_nav(shift, 0);
                HandleResult::Consumed
            }
            KeyCode::End => {
                self.handle_nav(shift, self.char_count());
                HandleResult::Consumed
            }
            KeyCode::Up => {
                if self.sidekick_visible {
                    self.sidekick_select_prev();
                } else {
                    self.handle_history_up();
                }
                HandleResult::Consumed
            }
            KeyCode::Down => {
                if self.sidekick_visible {
                    self.sidekick_select_next();
                } else {
                    self.handle_history_down();
                }
                HandleResult::Consumed
            }
            KeyCode::Tab => {
                self.try_complete();
                HandleResult::Consumed
            }
            KeyCode::Enter => {
                if self.sidekick_visible {
                    self.apply_sidekick_selection();
                }
                self.hide_sidekick();
                self.push_history();
                self.state
                    .put_command(self.submit_command, Some(Box::new(self.text.clone())));
                HandleResult::Consumed
            }
            KeyCode::Esc => {
                self.hide_sidekick();
                self.state.put_command(CM_CANCEL, None);
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }

    fn handle_char_key(&mut self, ch: char, key: &txv_core::event::KeyEvent, shift: bool) -> HandleResult {
        if key.modifiers.alt {
            match ch {
                'f' => {
                    let new = self.word_forward();
                    self.handle_nav(shift, new);
                }
                'b' => {
                    let new = self.word_backward();
                    self.handle_nav(shift, new);
                }
                'd' => {
                    let killed = self.kill_word_forward();
                    if !killed.is_empty() {
                        self.state.put_command(CM_COPY_TO_CLIPBOARD, Some(Box::new(killed)));
                    }
                    self.update_completions();
                }
                _ => return HandleResult::Ignored,
            }
            return HandleResult::Consumed;
        }
        if key.modifiers.ctrl {
            match ch {
                'a' => self.handle_nav(shift, 0),
                'e' => self.handle_nav(shift, self.char_count()),
                'f' => {
                    let new = (self.cursor + 1).min(self.char_count());
                    self.handle_nav(shift, new);
                }
                'b' => {
                    let new = self.cursor.saturating_sub(1);
                    self.handle_nav(shift, new);
                }
                'd' => {
                    self.handle_delete();
                    self.update_completions();
                }
                'k' => {
                    let killed = self.kill_to_end();
                    if !killed.is_empty() {
                        self.state.put_command(CM_COPY_TO_CLIPBOARD, Some(Box::new(killed)));
                    }
                    self.update_completions();
                }
                'u' => {
                    let killed = self.kill_to_start();
                    if !killed.is_empty() {
                        self.state.put_command(CM_COPY_TO_CLIPBOARD, Some(Box::new(killed)));
                    }
                    self.update_completions();
                }
                'w' => {
                    let killed = self.kill_word_back();
                    if !killed.is_empty() {
                        self.state.put_command(CM_COPY_TO_CLIPBOARD, Some(Box::new(killed)));
                    }
                    self.update_completions();
                }
                't' => self.transpose_chars(),
                'y' | 'v' => {
                    self.state.put_command(CM_PASTE_REQUEST, None);
                    return HandleResult::Consumed;
                }
                'c' => {
                    if let Some(text) = self.selected_text() {
                        self.state.put_command(CM_COPY_TO_CLIPBOARD, Some(Box::new(text)));
                    }
                    return HandleResult::Consumed;
                }
                _ => return HandleResult::Ignored,
            }
            return HandleResult::Consumed;
        }
        self.handle_char(ch);
        self.update_completions();
        HandleResult::Consumed
    }
}
