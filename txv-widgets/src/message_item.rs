//! MessageItem — displays status messages with timeout.

use std::time::Instant;

use txv_core::palette::palette;
use txv_core::prelude::*;
use txv_core::status::{ActiveItem, Gravity, VisibleItem};

/// Command ID for setting status message externally.
pub const CM_STATUS_MESSAGE: CommandId = 140;

pub struct MessageItem {
    display: String,
    style: Style,
    timeout_secs: u16,
    last_set: Option<Instant>,
    gravity: Gravity,
}

impl MessageItem {
    pub fn new(timeout_secs: u16) -> Self {
        Self {
            display: String::new(),
            style: Style::default(),
            timeout_secs,
            last_set: None,
            gravity: Gravity::Right,
        }
    }
    pub fn with_gravity(mut self, g: Gravity) -> Self {
        self.gravity = g;
        self
    }
}

impl ActiveItem for MessageItem {
    fn handle(&mut self, event: &Event, _sink: &EventSink) -> HandleResult {
        let Event::Command { id, data, .. } = event else {
            return HandleResult::Ignored;
        };
        if *id != CM_STATUS_MESSAGE {
            return HandleResult::Ignored;
        }
        let msg = data.as_ref().and_then(|b| b.downcast_ref::<Message>());
        let Some(msg) = msg else {
            return HandleResult::Ignored;
        };
        if msg.level() == MsgLevel::Debug {
            return HandleResult::Ignored;
        }
        self.display = if msg.count() > 1 {
            format!("[{}] {} (×{})", msg.origin(), msg.text(), msg.count())
        } else {
            format!("[{}] {}", msg.origin(), msg.text())
        };
        let pal = palette();
        self.style = match msg.level() {
            MsgLevel::Error => pal.style(StyleId::StateError),
            MsgLevel::Warn => pal.style(StyleId::StateWarning),
            _ => pal.style(StyleId::StateInfo),
        };
        self.last_set = Some(Instant::now());
        // Don't consume — let handler append to ring
        HandleResult::Ignored
    }
}

impl VisibleItem for MessageItem {
    fn label(&self) -> &str {
        &self.display
    }
    fn style(&self) -> Style {
        self.style
    }
    fn gravity(&self) -> Gravity {
        self.gravity
    }
    fn tick(&mut self) {
        if self.timeout_secs == 0 {
            return;
        }
        if let Some(set_at) = self.last_set {
            if set_at.elapsed().as_secs() >= u64::from(self.timeout_secs) {
                self.display.clear();
                self.last_set = None;
            }
        }
    }
}
