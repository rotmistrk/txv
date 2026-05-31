//! KeyLabelView — a status bar item that shows a label and intercepts a key.

use std::sync::Arc;

use txv_core::prelude::*;

/// A View-based status bar item that displays a label and emits a command on key press.
pub struct KeyLabelView {
    state: ViewState,
    key: KeyEvent,
    command: CommandId,
    data: Option<u16>,
    label_text: String,
    palette: Option<Arc<dyn Palette>>,
}

impl KeyLabelView {
    pub fn new(key: KeyEvent, command: CommandId, label: impl Into<String>) -> Self {
        let label_text = label.into();
        let mods = key.modifiers;
        let plain_char = !mods.ctrl && !mods.alt && !mods.shift;
        let char_len = label_text.chars().count();
        let display_len = if label_text.is_empty() {
            0
        } else if matches!(key.code, txv_core::event::KeyCode::Char(_)) && plain_char {
            char_len + 2 // "k:label"
        } else {
            char_len
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
            palette: None,
        }
    }

    pub fn with_data(mut self, data: u16) -> Self {
        self.data = Some(data);
        self
    }

    pub fn label(&self) -> &str {
        &self.label_text
    }

    fn resolve_style(&self, id: StyleId) -> Style {
        match &self.palette {
            Some(p) => p.style(id),
            None => txv_core::palette::palette().style(id),
        }
    }

    /// Format string for display: "k:label" where k is the key character (plain keys only).
    fn display_text(&self) -> String {
        if self.label_text.is_empty() {
            return String::new();
        }
        let mods = self.key.modifiers;
        let plain = !mods.ctrl && !mods.alt && !mods.shift;
        match self.key.code {
            txv_core::event::KeyCode::Char(c) if plain => format!("{c}:{}", self.label_text),
            _ => self.label_text.clone(),
        }
    }
}

impl View for KeyLabelView {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let style = self.resolve_style(StyleId::StatusBar);
        let text = self.display_text();
        let mods = self.key.modifiers;
        let plain = !mods.ctrl && !mods.alt && !mods.shift;
        let has_key_prefix = matches!(self.key.code, txv_core::event::KeyCode::Char(_)) && plain;
        let buf = self.state.buffer_mut();
        buf.fill(' ', style);
        if !text.is_empty() {
            let key_style = Style {
                attrs: Attrs {
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

    fn set_palette(&mut self, palette: Arc<dyn Palette>) {
        self.palette = Some(palette);
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
