//! Menu — modal popup menu.

use txv_core::prelude::*;

pub struct MenuItem {
    pub label: String,
    pub command: CommandId,
    pub enabled: bool,
}

impl MenuItem {
    pub fn new(label: impl Into<String>, command: CommandId) -> Self {
        Self {
            label: label.into(),
            command,
            enabled: true,
        }
    }
}

pub struct Menu {
    state: ViewState,
    pub items: Vec<MenuItem>,
    pub cursor: usize,
}

impl Menu {
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            state: ViewState::new(ViewOptions {
                modal: true,
                focusable: true,
                ..ViewOptions::default()
            }),
            items,
            cursor: 0,
        }
    }
}

impl View for Menu {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let pal = txv_core::palette::palette();
        let normal = Style::default();
        let selected = pal.interactive.cursor_focused.to_style();
        let disabled = pal.interactive.disabled.to_style();

        // Draw border
        let g = glyphs();
        let bx = &g.box_drawing;
        self.state.buffer_mut().hline(0, 0, w, bx.h, normal);
        self.state.buffer_mut().hline(0, h.saturating_sub(1), w, bx.h, normal);
        for row in 1..h.saturating_sub(1) {
            self.state.buffer_mut().put(0, row, bx.v, normal);
            self.state.buffer_mut().put(w.saturating_sub(1), row, bx.v, normal);
        }
        self.state.buffer_mut().put(0, 0, bx.tl, normal);
        self.state.buffer_mut().put(w.saturating_sub(1), 0, bx.tr, normal);
        self.state.buffer_mut().put(0, h.saturating_sub(1), bx.bl, normal);
        self.state
            .buffer_mut()
            .put(w.saturating_sub(1), h.saturating_sub(1), bx.br, normal);

        // Draw items
        let inner_w = w.saturating_sub(2);
        for (i, item) in self.items.iter().enumerate() {
            let row = i as u16 + 1;
            if row >= h.saturating_sub(1) {
                break;
            }
            let style = if !item.enabled {
                disabled
            } else if i == self.cursor {
                selected
            } else {
                normal
            };
            self.state.buffer_mut().hline(1, row, inner_w, ' ', style);
            self.state.buffer_mut().print(2, row, &item.label, style);
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Consumed; // modal captures all
        };
        match key.code {
            KeyCode::Up => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.state.mark_dirty();
                }
                HandleResult::Consumed
            }
            KeyCode::Down => {
                if self.cursor + 1 < self.items.len() {
                    self.cursor += 1;
                    self.state.mark_dirty();
                }
                HandleResult::Consumed
            }
            KeyCode::Enter => {
                if let Some(item) = self.items.get(self.cursor) {
                    if item.enabled {
                        self.state.put_command(item.command, None);
                    }
                }
                HandleResult::Consumed
            }
            KeyCode::Esc => {
                self.state.put_command(CM_CANCEL, None);
                HandleResult::Consumed
            }
            _ => HandleResult::Consumed, // modal swallows all keys
        }
    }
}
