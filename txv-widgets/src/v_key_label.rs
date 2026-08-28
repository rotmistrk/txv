//! KeyLabelView — a status bar item that shows a label and intercepts a key.

use std::any::Any;
use std::sync::Arc;

use txv_core::command_registry;
use txv_core::prelude::*;

/// A View-based status bar item that displays a label and emits a command on key press.
pub struct KeyLabelView {
    state: ViewState,
    key: KeyEvent,
    command: CommandId,
    data: Option<u16>,
    /// Short text shown in status bar (e.g., "~F2~:Tree").
    label_text: String,
    /// Longer description for help system (e.g., "Focus tree panel").
    help_text: String,
    palette: Option<Arc<dyn Palette>>,
}

impl KeyLabelView {
    pub fn new(key: KeyEvent, command: CommandId, label: impl Into<String>) -> Self {
        let label_text = label.into();
        let mods = key.modifiers();
        let plain_char = !mods.ctrl() && !mods.alt() && !mods.shift();
        let display_len = display_width(&label_text, plain_char, key.code());
        let w = if display_len == 0 {
            0
        } else {
            display_len as u16 + 2
        };
        let mut state = ViewState::new(ViewOptions::default().with_preprocess());
        state.set_bounds(Rect::new(0, 0, w, 1));
        Self {
            state,
            key,
            command,
            data: None,
            label_text,
            help_text: String::new(),
            palette: None,
        }
    }

    pub fn with_data(mut self, data: u16) -> Self {
        self.data = Some(data);
        self
    }

    /// Set a separate help text for the F1 help system.
    /// If not set, uses label_text or falls back to command registry.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help_text = help.into();
        self
    }

    pub fn label(&self) -> &str {
        &self.label_text
    }

    delegate_palette!(palette);

    fn display_text(&self) -> String {
        if self.label_text.is_empty() {
            return String::new();
        }
        let mods = self.key.modifiers();
        let plain = !mods.ctrl() && !mods.alt() && !mods.shift();
        match self.key.code() {
            KeyCode::Char(c) if plain => format!("~{c}~:{}", self.label_text),
            _ => self.label_text.clone(),
        }
    }
}

fn display_width(label: &str, plain_char: bool, code: KeyCode) -> usize {
    if label.is_empty() {
        return 0;
    }
    let base = label.chars().filter(|c| *c != '~').count();
    if matches!(code, KeyCode::Char(_)) && plain_char {
        base + 2
    } else {
        base
    }
}

fn format_key_event(key: &KeyEvent) -> String {
    let mut s = String::new();
    let m = key.modifiers();
    if m.ctrl() {
        s.push_str("C-");
    }
    if m.alt() {
        s.push_str("M-");
    }
    if m.shift() {
        s.push_str("S-");
    }
    match key.code() {
        KeyCode::Char(c) => s.push(c),
        KeyCode::F(n) => s.push_str(&format!("F{n}")),
        KeyCode::Enter => s.push_str("Enter"),
        KeyCode::Esc => s.push_str("Esc"),
        KeyCode::Tab => s.push_str("Tab"),
        KeyCode::Up => s.push_str("Up"),
        KeyCode::Down => s.push_str("Down"),
        KeyCode::Left => s.push_str("Left"),
        KeyCode::Right => s.push_str("Right"),
        _ => s.push('?'),
    }
    s
}

impl View for KeyLabelView {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let style = self.resolve_style(StyleId::StatusBar);
        let text = self.display_text();
        let buf = self.state.buffer_mut();
        buf.fill(' ', style);
        if !text.is_empty() {
            Self::render_styled_text(buf, &text, style);
        }
    }

    fn set_palette(&mut self, palette: Arc<dyn Palette>) {
        self.palette = Some(palette);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Key(k) = event {
            if *k == self.key {
                let payload = self.data.map(|d| Box::new(d) as Box<dyn Any + Send>);
                self.state.put_command(self.command, payload);
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }

    fn key_help(&self) -> Vec<txv_core::key_help::KeyHelpEntry> {
        // Priority: explicit help_text > label_text > command registry
        let action = if !self.help_text.is_empty() {
            self.help_text.clone()
        } else if !self.label_text.is_empty() {
            self.label_text.replace('~', "")
        } else {
            command_registry::label(self.command)
        };
        vec![txv_core::key_help::KeyHelpEntry::new(
            format_key_event(&self.key),
            action,
            "Global",
        )]
    }
}

impl KeyLabelView {
    fn render_styled_text(buf: &mut Buffer, text: &str, style: Style) {
        let bold = style.with_attrs(style.attrs().bold());
        let mut x: u16 = 1;
        let mut in_key = false;
        let mut chars = text.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch == '~' {
                if chars.peek() == Some(&'~') {
                    chars.next();
                } else {
                    in_key = !in_key;
                    continue;
                }
            }
            let s = if in_key {
                bold
            } else {
                style
            };
            buf.put(x, 0, ch, s);
            x += 1;
        }
    }
}
