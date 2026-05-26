//! KeyLabelView — a status bar item that shows a label and intercepts a key.

use txv_core::prelude::*;

/// A View-based status bar item that displays a label and emits a command on key press.
pub struct KeyLabelView {
    state: ViewState,
    key: KeyEvent,
    command: CommandId,
    data: Option<u16>,
    label_text: String,
}

impl KeyLabelView {
    pub fn new(key: KeyEvent, command: CommandId, label: impl Into<String>) -> Self {
        let label_text = label.into();
        let w = if label_text.is_empty() {
            0
        } else {
            label_text.len() as u16 + 2
        };
        let mut state = ViewState::new(ViewOptions {
            preprocess: true,
            focusable: false,
            ..ViewOptions::default()
        });
        state.set_bounds(Rect { x: 0, y: 0, w, h: 1 });
        Self {
            state,
            key,
            command,
            data: None,
            label_text,
        }
    }

    pub fn with_data(mut self, data: u16) -> Self {
        self.data = Some(data);
        self
    }

    pub fn label(&self) -> &str {
        &self.label_text
    }
}

impl View for KeyLabelView {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let buf = self.state.buffer_mut();
        let style = Style {
            attrs: Attrs {
                reverse: true,
                ..Attrs::default()
            },
            ..Style::default()
        };
        buf.fill(' ', style);
        if !self.label_text.is_empty() {
            buf.print(1, 0, &self.label_text, style);
        }
        self.state.mark_redrawn();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Key(k) = event {
            if *k == self.key {
                let payload = self.data.map(|d| Box::new(d) as Box<dyn std::any::Any + Send>);
                self.state.put_command(self.command, payload);
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }
}
