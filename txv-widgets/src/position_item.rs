//! PositionItem — displays cursor position as "Ln N, Col M".

use txv_core::prelude::*;
use txv_core::status::{ActiveItem, Gravity, VisibleItem};

use crate::cursor_pos::CursorPos;

/// Displays cursor position as "Ln N, Col M".
pub struct PositionItem {
    command_id: CommandId,
    label_text: String,
}

impl PositionItem {
    pub fn new(command_id: CommandId) -> Self {
        Self {
            command_id,
            label_text: "Ln 1, Col 1".to_string(),
        }
    }
}

impl ActiveItem for PositionItem {
    fn handle(&mut self, event: &Event, _sink: &EventSink) -> HandleResult {
        let Event::Command { id, data, .. } = event else {
            return HandleResult::Ignored;
        };
        if *id != self.command_id {
            return HandleResult::Ignored;
        }
        let pos = data.as_ref().and_then(|b| b.downcast_ref::<CursorPos>());
        let Some(pos) = pos else {
            return HandleResult::Ignored;
        };
        self.label_text = format!("Ln {}, Col {}", pos.line(), pos.col());
        HandleResult::Consumed
    }
}

impl VisibleItem for PositionItem {
    fn label(&self) -> &str {
        &self.label_text
    }
    fn gravity(&self) -> Gravity {
        Gravity::Right
    }
}
