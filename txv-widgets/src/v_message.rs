//! MessageView — shows toast messages, auto-dismisses after timeout.

use std::any::Any;
use std::time::Instant;
use txv_core::prelude::*;

use crate::focus_gated_group::{CM_ACTIVATE_GROUP, CM_DEACTIVATE_GROUP};
use crate::resize_helpers::resize_width_to;
use crate::status_items::CM_STATUS_MESSAGE;

/// Group id that ModalKey broadcasts to every status item when a prompt opens/closes.
const ALL_GROUPS: u16 = u16::MAX;

/// A View-based status bar item that displays transient messages.
///
/// While a modal prompt (M-x, file finder) is open, the message yields its
/// width to the prompt; it keeps the text and reappears when the prompt closes.
pub struct MessageView {
    state: ViewState,
    display: String,
    display_style: Style,
    timeout_secs: u16,
    last_set: Option<Instant>,
    yielded: bool,
}

impl MessageView {
    pub fn new(timeout_secs: u16) -> Self {
        let mut state = ViewState::new(ViewOptions::default().with_preprocess());
        state.set_bounds(Rect::new(0, 0, 0, 1));
        Self {
            state,
            display: String::new(),
            display_style: Style::default(),
            timeout_secs,
            last_set: None,
            yielded: false,
        }
    }

    fn update_bounds(&mut self) {
        let w = if self.display.is_empty() || self.yielded {
            0
        } else {
            self.display.len() as u16 + 2
        };
        resize_width_to(&mut self.state, w);
    }

    fn apply_message(&mut self, msg: &Message) {
        if msg.level() == MsgLevel::Debug {
            return;
        }
        self.display = if msg.count() > 1 {
            format!("[{}] {} (×{})", msg.origin(), msg.text(), msg.count())
        } else {
            format!("[{}] {}", msg.origin(), msg.text())
        };
        let pal = palette();
        let bar_bg = pal.style(StyleId::StatusBar).bg();
        let fg_style = match msg.level() {
            MsgLevel::Error => pal.style(StyleId::StateError),
            MsgLevel::Warn => pal.style(StyleId::StateWarning),
            _ => pal.style(StyleId::StateInfo),
        };
        self.display_style = Style::new(fg_style.fg(), bar_bg);
        self.last_set = Some(Instant::now());
        self.update_bounds();
        self.state.mark_dirty();
    }

    /// React to ModalKey's wildcard group broadcast: yield while a prompt is open.
    fn on_prompt_toggle(&mut self, id: CommandId, data: &Option<Box<dyn Any + Send>>) {
        let is_wildcard = data.as_ref().and_then(|d| d.downcast_ref::<u16>()) == Some(&ALL_GROUPS);
        if !is_wildcard {
            return;
        }
        self.yielded = id == CM_DEACTIVATE_GROUP;
        self.update_bounds();
        self.state.mark_dirty();
    }

    fn check_timeout(&mut self) {
        if self.timeout_secs == 0 {
            return;
        }
        if let Some(set_at) = self.last_set {
            if set_at.elapsed().as_secs() >= u64::from(self.timeout_secs) {
                self.display.clear();
                self.last_set = None;
                self.update_bounds();
                self.state.mark_dirty();
            }
        }
    }
}

impl View for MessageView {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let buf = self.state.buffer_mut();
        if self.display.is_empty() {
            let style = palette().style(StyleId::StatusBar);
            buf.fill(' ', style);
            return;
        }
        buf.fill(' ', self.display_style);
        buf.print(1, 0, &self.display, self.display_style);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        match event {
            Event::Command { id, data, .. } if *id == CM_STATUS_MESSAGE => {
                if let Some(msg) = data.as_ref().and_then(|b| b.downcast_ref::<Message>()) {
                    self.apply_message(msg);
                }
                HandleResult::Ignored
            }
            Event::Command { id, data, .. } if *id == CM_DEACTIVATE_GROUP || *id == CM_ACTIVATE_GROUP => {
                self.on_prompt_toggle(*id, data);
                HandleResult::Ignored
            }
            Event::Tick => {
                self.check_timeout();
                HandleResult::Ignored
            }
            _ => HandleResult::Ignored,
        }
    }
}

#[cfg(test)]
#[path = "v_message_tests.rs"]
mod tests;
