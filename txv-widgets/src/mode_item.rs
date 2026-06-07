//! ModeItem — displays the current editor mode.

use txv_core::prelude::*;
use txv_core::status::{ActiveItem, Gravity, VisibleItem};

/// Displays the current editor mode (NOR, INS, VIS, CMD).
pub struct ModeItem {
    command_id: CommandId,
    label_text: String,
}

impl ModeItem {
    pub fn new(command_id: CommandId) -> Self {
        Self {
            command_id,
            label_text: "NOR".to_string(),
        }
    }
}

impl ActiveItem for ModeItem {
    fn handle(&mut self, event: &Event, _sink: &EventSink) -> HandleResult {
        let Event::Command { id, data, .. } = event else {
            return HandleResult::Ignored;
        };
        if *id != self.command_id {
            return HandleResult::Ignored;
        }
        let mode = data.as_ref().and_then(|b| b.downcast_ref::<String>());
        let Some(mode) = mode else {
            return HandleResult::Ignored;
        };
        self.label_text = mode.clone();
        HandleResult::Consumed
    }
}

impl VisibleItem for ModeItem {
    fn label(&self) -> &str {
        &self.label_text
    }
    fn gravity(&self) -> Gravity {
        Gravity::Right
    }
}
