//! ConfirmItem — status bar item for single-key confirmation prompts.
//!
//! Activates on a command, shows a prompt, and emits a response command
//! with the key pressed (as a char). Deactivates after one key.

use txv_core::cell::Style;
use txv_core::palette::{palette, StyleId};
use txv_core::prelude::*;
use txv_core::status::{ActiveItem, Gravity, VisibleItem};

/// Command ID that activates this item (payload: String prompt text).
pub struct ConfirmItem {
    activate_command: CommandId,
    response_command: CommandId,
    active: bool,
    label: String,
    highlight_pos: usize,
    tick_counter: u8,
}

impl ConfirmItem {
    pub fn new(activate_command: CommandId, response_command: CommandId) -> Self {
        Self {
            activate_command,
            response_command,
            active: false,
            label: String::new(),
            highlight_pos: 0,
            tick_counter: 0,
        }
    }
}

impl ActiveItem for ConfirmItem {
    fn handle(&mut self, event: &Event, sink: &EventSink) -> HandleResult {
        if !self.active {
            return self.try_activate(event);
        }
        self.handle_active_key(event, sink)
    }

    fn is_exclusive(&self) -> bool {
        self.active
    }
}

impl ConfirmItem {
    fn try_activate(&mut self, event: &Event) -> HandleResult {
        let Event::Command { id, data, .. } = event else {
            return HandleResult::Ignored;
        };
        if *id != self.activate_command {
            return HandleResult::Ignored;
        }
        let text = data.as_ref().and_then(|b| b.downcast_ref::<String>());
        let Some(text) = text else {
            return HandleResult::Ignored;
        };
        self.label = text.clone();
        self.active = true;
        self.highlight_pos = 0;
        self.tick_counter = 0;
        HandleResult::Consumed
    }

    fn handle_active_key(&mut self, event: &Event, sink: &EventSink) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Consumed;
        };
        let ch = match key.code() {
            KeyCode::Char(c) => c,
            KeyCode::Esc => 'c',
            _ => return HandleResult::Consumed,
        };
        self.active = false;
        self.label.clear();
        sink.push_command(self.response_command, Some(Box::new(ch)));
        HandleResult::Consumed
    }
}

impl VisibleItem for ConfirmItem {
    fn label(&self) -> &str {
        &self.label
    }

    fn gravity(&self) -> Gravity {
        Gravity::Left
    }

    fn style(&self) -> Style {
        if self.active {
            palette().style(StyleId::StatusQuestion)
        } else {
            Style::default()
        }
    }

    fn highlight_offset(&self) -> Option<usize> {
        if self.active && !self.label.is_empty() {
            Some(self.highlight_pos)
        } else {
            None
        }
    }

    fn tick(&mut self) {
        if !self.active || self.label.is_empty() {
            return;
        }
        // Advance highlight at ~2 chars/sec (tick is ~100ms, so every 5 ticks)
        self.tick_counter += 1;
        if self.tick_counter >= 5 {
            self.tick_counter = 0;
            self.highlight_pos += 1;
            if self.highlight_pos >= self.label.len() {
                self.highlight_pos = 0;
            }
        }
    }
}
