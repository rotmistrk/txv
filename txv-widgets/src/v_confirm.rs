//! ConfirmView — Yes/No confirmation prompt as a proper View.

use std::sync::Arc;

use txv_core::prelude::*;

/// A View-based confirmation prompt for the status bar.
pub struct ConfirmView {
    state: ViewState,
    palette: Option<Arc<dyn Palette>>,
    activate_command: CommandId,
    response_command: CommandId,
    active: bool,
    prompt: String,
    highlight_pos: usize,
    tick_counter: u8,
}

impl ConfirmView {
    pub fn new(activate_command: CommandId, response_command: CommandId) -> Self {
        let mut state = ViewState::new(ViewOptions {
            preprocess: true,
            focusable: false,
            ..ViewOptions::default()
        });
        state.set_bounds(Rect { x: 0, y: 0, w: 0, h: 1 });
        Self {
            state,
            palette: None,
            activate_command,
            response_command,
            active: false,
            prompt: String::new(),
            highlight_pos: 0,
            tick_counter: 0,
        }
    }

    fn resolve_style(&self, id: StyleId) -> Style {
        match &self.palette {
            Some(p) => p.style(id),
            None => txv_core::palette::palette().style(id),
        }
    }

    fn display_text(&self) -> String {
        if self.active {
            format!("{} [y/n]", self.prompt)
        } else {
            String::new()
        }
    }

    fn update_bounds(&mut self) {
        let label = self.display_text();
        let w = if label.is_empty() {
            0
        } else {
            label.len() as u16 + 2
        };
        let bounds = self.state.bounds();
        if bounds.w != w {
            self.state.set_bounds(Rect {
                x: bounds.x,
                y: bounds.y,
                w,
                h: 1,
            });
        }
    }

    fn respond(&mut self, ch: char) {
        self.active = false;
        self.prompt.clear();
        self.state.put_command(self.response_command, Some(Box::new(ch)));
        self.update_bounds();
        self.state.mark_dirty();
    }

    fn try_activate(&mut self, event: &Event) -> HandleResult {
        let Event::Command { id, data } = event else {
            return HandleResult::Ignored;
        };
        if *id != self.activate_command {
            return HandleResult::Ignored;
        }
        let prompt = data.as_ref().and_then(|d| d.downcast_ref::<String>()).cloned();
        let Some(text) = prompt else {
            return HandleResult::Ignored;
        };
        self.prompt = text;
        self.active = true;
        self.highlight_pos = 0;
        self.tick_counter = 0;
        self.update_bounds();
        self.state.mark_dirty();
        HandleResult::Consumed
    }
}

impl View for ConfirmView {
    delegate_view_state!(state, override { options });

    fn options(&self) -> ViewOptions {
        ViewOptions {
            preprocess: true,
            focusable: false,
            modal: self.active,
            ..ViewOptions::default()
        }
    }

    fn draw(&mut self) {
        let bar_style = self.resolve_style(StyleId::StatusBar);
        let q = self.resolve_style(StyleId::StatusQuestion);
        let h = self.resolve_style(StyleId::StatusHighlight);
        let prompt_style = Style { bg: bar_style.bg, ..q };
        let hi_style = Style { bg: bar_style.bg, ..h };
        let label = self.display_text();
        let highlight_pos = self.highlight_pos;
        let buf = self.state.buffer_mut();
        buf.fill(' ', bar_style);
        if !self.active {
            self.state.mark_redrawn();
            return;
        }
        let w = buf.width() as usize;
        for (i, ch) in label.chars().enumerate() {
            if i + 1 >= w {
                break;
            }
            let s = if i == highlight_pos {
                hi_style
            } else {
                prompt_style
            };
            buf.put((i + 1) as u16, 0, ch, s);
        }
        self.state.mark_redrawn();
    }

    fn set_palette(&mut self, palette: Arc<dyn Palette>) {
        self.palette = Some(palette);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Tick = event {
            if self.active {
                self.tick_counter += 1;
                if self.tick_counter >= 5 {
                    self.tick_counter = 0;
                    let len = self.display_text().len();
                    if len > 0 {
                        self.highlight_pos = (self.highlight_pos + 1) % len;
                    }
                    self.state.mark_dirty();
                }
            }
            return HandleResult::Ignored;
        }
        if !self.active {
            return self.try_activate(event);
        }
        let Event::Key(key) = event else {
            return HandleResult::Consumed;
        };
        match key.code {
            KeyCode::Char(ch) => self.respond(ch),
            KeyCode::Esc => self.respond('c'),
            _ => {}
        }
        HandleResult::Consumed
    }
}
