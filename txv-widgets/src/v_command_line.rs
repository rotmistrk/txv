//! CommandLineView — input line with completion, as a proper View.

use txv_core::prelude::*;

/// A View-based command input line for the status bar.
pub struct CommandLineView {
    state: ViewState,
    activation_keys: Vec<KeyEvent>,
    command_id: CommandId,
    prefill_command_id: Option<CommandId>,
    active: bool,
    text: String,
    cursor: usize,
    completer: Option<Box<dyn Completer>>,
    dormant_label: String,
}

impl CommandLineView {
    pub fn new(keys: &[KeyEvent], command_id: CommandId) -> Self {
        let mut state = ViewState::new(ViewOptions {
            preprocess: true,
            focusable: false,
            ..ViewOptions::default()
        });
        state.set_bounds(Rect { x: 0, y: 0, w: 0, h: 1 });
        Self {
            state,
            activation_keys: keys.to_vec(),
            command_id,
            prefill_command_id: None,
            active: false,
            text: String::new(),
            cursor: 0,
            completer: None,
            dormant_label: String::new(),
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.dormant_label = label.into();
        self.update_bounds();
        self
    }

    pub fn with_completer(mut self, c: Box<dyn Completer>) -> Self {
        self.completer = Some(c);
        self
    }

    pub fn with_prefill_command(mut self, id: CommandId) -> Self {
        self.prefill_command_id = Some(id);
        self
    }

    pub fn set_completer(&mut self, c: Box<dyn Completer>) {
        self.completer = Some(c);
    }

    fn activate(&mut self) {
        self.active = true;
        self.text.clear();
        self.cursor = 0;
        self.update_bounds();
        self.state.mark_dirty();
    }

    fn deactivate(&mut self) {
        self.active = false;
        self.text.clear();
        self.cursor = 0;
        self.update_bounds();
        self.state.mark_dirty();
    }

    fn display_text(&self) -> String {
        if self.active {
            format!(":{}", self.text)
        } else {
            self.dormant_label.clone()
        }
    }

    fn update_bounds(&mut self) {
        let label = self.display_text();
        let w = if label.is_empty() {
            0
        } else {
            label.len() as u16 + 2
        };
        let bounds = self.state.bounds();
        if bounds.w != w {
            self.state.set_bounds(Rect {
                x: bounds.x,
                y: bounds.y,
                w,
                h: 1,
            });
        }
    }

    fn try_complete(&mut self) {
        if let Some(ref completer) = self.completer {
            let mut first: Option<String> = None;
            let mut count = 0u32;
            let _ = completer.complete(&self.text, self.cursor, &mut |c| {
                count += 1;
                if count == 1 {
                    first = Some(c.text().to_string());
                }
                Ok(count < 2)
            });
            if count == 1 {
                if let Some(text) = first {
                    self.text = text;
                    self.cursor = self.text.len();
                    self.update_bounds();
                    self.state.mark_dirty();
                }
            }
        }
    }

    fn handle_inactive(&mut self, event: &Event) -> HandleResult {
        if let Event::Key(k) = event {
            if self.activation_keys.contains(k) {
                self.activate();
                return HandleResult::Consumed;
            }
        }
        if let Event::Command { id, data } = event {
            if Some(*id) == self.prefill_command_id {
                if let Some(prefix) = data.as_ref().and_then(|b| b.downcast_ref::<String>()) {
                    self.activate();
                    self.text = prefix.clone();
                    self.cursor = self.text.len();
                    self.update_bounds();
                    self.state.mark_dirty();
                    return HandleResult::Consumed;
                }
            }
        }
        HandleResult::Ignored
    }

    fn handle_active_key(&mut self, key: &KeyEvent) -> HandleResult {
        match &key.code {
            KeyCode::Esc => self.deactivate(),
            KeyCode::Enter => {
                let cmd = self.text.clone();
                self.deactivate();
                if !cmd.is_empty() {
                    self.state.put_command(self.command_id, Some(Box::new(cmd)));
                }
            }
            KeyCode::Tab => self.try_complete(),
            KeyCode::Backspace => self.handle_backspace(),
            KeyCode::Left => self.move_cursor_left(),
            KeyCode::Right => self.move_cursor_right(),
            KeyCode::Char(ch) => {
                self.text.insert(self.cursor, *ch);
                self.cursor += ch.len_utf8();
                self.update_bounds();
                self.state.mark_dirty();
            }
            _ => {}
        }
        HandleResult::Consumed
    }

    fn handle_backspace(&mut self) {
        if self.cursor > 0 {
            let prev = self.text[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.text.remove(prev);
            self.cursor = prev;
            self.update_bounds();
            self.state.mark_dirty();
        } else {
            self.deactivate();
        }
    }

    fn move_cursor_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.text[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor = self.text[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.text.len());
        }
    }
}

impl View for CommandLineView {
    delegate_view_state!(state, override { options });

    fn options(&self) -> ViewOptions {
        ViewOptions {
            preprocess: true,
            focusable: false,
            modal: self.active,
            ..ViewOptions::default()
        }
    }

    fn draw(&mut self) {
        let label = self.display_text();
        let style = txv_core::palette::palette().chrome().status_bar();
        let buf = self.state.buffer_mut();
        buf.fill(' ', style);
        if !label.is_empty() {
            buf.print(1, 0, &label, style);
        }
        self.state.mark_redrawn();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if !self.active {
            return self.handle_inactive(event);
        }
        let Event::Key(key) = event else {
            return HandleResult::Consumed;
        };
        self.handle_active_key(key)
    }
}
