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
        let display_len = if label_text.is_empty() {
            0
        } else {
            match key.code {
                txv_core::event::KeyCode::Char(_) => label_text.len() + 2, // "k:label"
                _ => label_text.len(),
            }
        };
        let w = if display_len == 0 {
            0
        } else {
            display_len as u16 + 2
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

    /// Format string for display: "k:label" where k is the key character.
    fn display_text(&self) -> String {
        if self.label_text.is_empty() {
            return String::new();
        }
        match self.key.code {
            txv_core::event::KeyCode::Char(c) => format!("{c}:{}", self.label_text),
            _ => self.label_text.clone(),
        }
    }
}

impl View for KeyLabelView {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let style = txv_core::palette::palette().chrome().status_bar();
        let text = self.display_text();
        let has_key_prefix = matches!(self.key.code, txv_core::event::KeyCode::Char(_));
        let buf = self.state.buffer_mut();
        buf.fill(' ', style);
        if !text.is_empty() {
            let key_style = Style {
                attrs: txv_core::cell::Attrs {
                    bold: true,
                    ..style.attrs
                },
                ..style
            };
            if has_key_prefix {
                buf.print(1, 0, &text[..2], key_style);
                buf.print(3, 0, &text[2..], style);
            } else {
                buf.print(1, 0, &text, style);
            }
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
