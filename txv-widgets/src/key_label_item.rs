//! KeyLabelItem — key binding that emits a command.

use std::any::Any;

use txv_core::prelude::*;
use txv_core::status::{ActiveItem, Gravity, VisibleItem};

pub struct KeyLabelItem {
    key: KeyEvent,
    command: CommandId,
    data: Option<u16>,
    label_text: String,
    gravity: Gravity,
}

impl KeyLabelItem {
    pub fn new(key: KeyEvent, command: CommandId, label: impl Into<String>) -> Self {
        Self {
            key,
            command,
            data: None,
            label_text: label.into(),
            gravity: Gravity::Left,
        }
    }
    pub fn hidden(key: KeyEvent, command: CommandId) -> Self {
        Self {
            key,
            command,
            data: None,
            label_text: String::new(),
            gravity: Gravity::Left,
        }
    }
    pub fn hidden_with_data(key: KeyEvent, command: CommandId, data: u16) -> Self {
        Self {
            key,
            command,
            data: Some(data),
            label_text: String::new(),
            gravity: Gravity::Left,
        }
    }
    pub fn with_gravity(mut self, g: Gravity) -> Self {
        self.gravity = g;
        self
    }
}

impl ActiveItem for KeyLabelItem {
    fn handle(&mut self, event: &Event, sink: &EventSink) -> HandleResult {
        if let Event::Key(k) = event {
            if *k == self.key {
                let payload = self.data.map(|d| Box::new(d) as Box<dyn Any + Send>);
                sink.push_command(self.command, payload);
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }
}

impl VisibleItem for KeyLabelItem {
    fn label(&self) -> &str {
        &self.label_text
    }
    fn gravity(&self) -> Gravity {
        self.gravity
    }
}
