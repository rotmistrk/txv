//! ConfirmItem — status bar item for single-key confirmation prompts.
//!
//! Activates on a command, shows a prompt, and emits a response command
//! with the key pressed (as a char). Deactivates after one key.

use txv_core::prelude::*;
use txv_core::status::{ActiveItem, Gravity, VisibleItem};

/// Command ID that activates this item (payload: String prompt text).
pub struct ConfirmItem {
    activate_command: CommandId,
    response_command: CommandId,
    active: bool,
    label: String,
}

impl ConfirmItem {
    pub fn new(activate_command: CommandId, response_command: CommandId) -> Self {
        Self {
            activate_command,
            response_command,
            active: false,
            label: String::new(),
        }
    }
}

impl ActiveItem for ConfirmItem {
    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        if !self.active {
            if let Event::Command { id, data } = event {
                if *id == self.activate_command {
                    if let Some(boxed) = data.as_ref() {
                        if let Some(text) = boxed.downcast_ref::<String>() {
                            self.label = text.clone();
                            self.active = true;
                            return HandleResult::Consumed;
                        }
                    }
                }
            }
            return HandleResult::Ignored;
        }
        // Active: consume the next key and emit response
        if let Event::Key(key) = event {
            let ch = match key.code {
                KeyCode::Char(c) => c,
                KeyCode::Esc => 'c',
                _ => return HandleResult::Consumed,
            };
            self.active = false;
            self.label.clear();
            queue.put_command(self.response_command, Some(Box::new(ch)));
            return HandleResult::Consumed;
        }
        HandleResult::Consumed
    }

    fn is_exclusive(&self) -> bool {
        self.active
    }
}

impl VisibleItem for ConfirmItem {
    fn label(&self) -> &str {
        &self.label
    }

    fn gravity(&self) -> Gravity {
        Gravity::Left
    }
}
