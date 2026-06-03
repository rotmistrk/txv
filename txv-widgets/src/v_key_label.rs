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
        // Display length excludes ~ markers (style toggles)
        let display_len = display_width(&label_text, plain_char, key.code);
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
            txv_core::event::KeyCode::Char(c) if plain => format!("~{c}~:{}", self.label_text),
            _ => self.label_text.clone(),
        }
    }
}

/// Compute visible width of label, excluding ~ style markers.
fn display_width(label: &str, plain_char: bool, code: txv_core::event::KeyCode) -> usize {
    if label.is_empty() {
        return 0;
    }
    let base = label.chars().filter(|c| *c != '~').count();
    if matches!(code, txv_core::event::KeyCode::Char(_)) && plain_char {
        base + 2 // "k:label"
    } else {
        base
    }
}

impl View for KeyLabelView {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let style = self.resolve_style(StyleId::StatusBar);
        let text = self.display_text();
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
            // Render with ~ as style toggle: ~text~ renders in key_style
            let mut x: u16 = 1;
            let mut in_key = false;
            let mut chars = text.chars().peekable();
            while let Some(ch) = chars.next() {
                if ch == '~' {
                    if chars.peek() == Some(&'~') {
                        chars.next();
                        buf.put(
                            x,
                            0,
                            '~',
                            if in_key {
                                key_style
                            } else {
                                style
                            },
                        );
                        x += 1;
                    } else {
                        in_key = !in_key;
                    }
                } else {
                    buf.put(
                        x,
                        0,
                        ch,
                        if in_key {
                            key_style
                        } else {
                            style
                        },
                    );
                    x += 1;
                }
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
