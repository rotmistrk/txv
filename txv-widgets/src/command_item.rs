//! CommandItem — status bar item for command input with completion.

use txv_core::prelude::*;
use txv_core::status::{ActiveItem, Gravity, VisibleItem};

pub struct CommandItem {
    activation_keys: Vec<KeyEvent>,
    command_id: CommandId,
    prefill_command_id: Option<CommandId>,
    active: bool,
    text: String,
    cursor: usize,
    completer: Option<Box<dyn Completer>>,
    label_text: String,
    dormant_label: String,
    gravity: Gravity,
}

impl CommandItem {
    pub fn new(keys: &[KeyEvent], command_id: CommandId) -> Self {
        Self {
            activation_keys: keys.to_vec(),
            active: false,
            command_id,
            prefill_command_id: None,
            text: String::new(),
            cursor: 0,
            completer: None,
            label_text: String::new(),
            dormant_label: String::new(),
            gravity: Gravity::Left,
        }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.dormant_label = label.into();
        self.label_text = self.dormant_label.clone();
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
        self.label_text = ":".to_string();
    }

    fn deactivate(&mut self) {
        self.active = false;
        self.text.clear();
        self.cursor = 0;
        self.label_text = self.dormant_label.clone();
    }

    fn update_label(&mut self) {
        self.label_text = format!(":{}", self.text);
    }

    fn try_complete(&mut self) {
        if let Some(ref completer) = self.completer {
            let completions = completer.complete(&self.text, self.cursor);
            if completions.len() == 1 {
                self.text = completions[0].text().to_string();
                self.cursor = self.text.len();
                self.update_label();
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
                    self.update_label();
                    return HandleResult::Consumed;
                }
            }
        }
        HandleResult::Ignored
    }

    fn handle_active_key(&mut self, key: &KeyEvent, sink: &EventSink) {
        match &key.code {
            KeyCode::Esc => self.deactivate(),
            KeyCode::Enter => {
                let cmd = self.text.clone();
                self.deactivate();
                if !cmd.is_empty() {
                    sink.push_command(self.command_id, Some(Box::new(cmd)));
                }
            }
            KeyCode::Tab => self.try_complete(),
            KeyCode::Backspace => self.handle_backspace(),
            KeyCode::Left => self.move_cursor_left(),
            KeyCode::Right => self.move_cursor_right(),
            KeyCode::Char(ch) => {
                self.text.insert(self.cursor, *ch);
                self.cursor += ch.len_utf8();
                self.update_label();
            }
            _ => {}
        }
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
            self.update_label();
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

impl ActiveItem for CommandItem {
    fn handle(&mut self, event: &Event, sink: &EventSink) -> HandleResult {
        if !self.active {
            return self.handle_inactive(event);
        }
        let Event::Key(key) = event else {
            return HandleResult::Consumed;
        };
        self.handle_active_key(key, sink);
        HandleResult::Consumed
    }

    fn is_exclusive(&self) -> bool {
        self.active
    }
}

impl VisibleItem for CommandItem {
    fn label(&self) -> &str {
        &self.label_text
    }

    fn gravity(&self) -> Gravity {
        self.gravity
    }
}
