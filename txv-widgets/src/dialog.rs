//! Dialog — modal dialog with message and OK/Cancel buttons.

use txv_core::prelude::*;

pub struct Dialog {
    state: ViewState,
    pub title_text: String,
    pub message: String,
    pub buttons: Vec<String>,
    pub focused_button: usize,
}

impl Dialog {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        let mut s = Self {
            state: ViewState::new(ViewOptions {
                modal: true,
                focusable: true,
                ..ViewOptions::default()
            }),
            title_text: title.into(),
            message: message.into(),
            buttons: vec!["OK".into(), "Cancel".into()],
            focused_button: 0,
        };
        s.state.set_title(s.title_text.clone());
        s
    }

    pub fn set_buttons(&mut self, buttons: Vec<String>) {
        self.buttons = buttons;
        self.focused_button = 0;
        self.state.mark_dirty();
    }
}

impl View for Dialog {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let pal = txv_core::palette::palette();
        let normal = Style::default();
        let border_style = pal.base().border();
        let btn_normal = Style::default();
        let btn_focused = pal.interactive().input_cursor();

        // Fill background
        for row in 0..h {
            self.state.buffer_mut().hline(0, row, w, ' ', normal);
        }

        // Border
        let g = glyphs();
        let bx = &g.box_drawing;
        self.state.buffer_mut().hline(0, 0, w, bx.h_heavy, border_style);
        self.state
            .buffer_mut()
            .hline(0, h.saturating_sub(1), w, bx.h_heavy, border_style);
        for row in 1..h.saturating_sub(1) {
            self.state.buffer_mut().put(0, row, bx.v_heavy, border_style);
            self.state
                .buffer_mut()
                .put(w.saturating_sub(1), row, bx.v_heavy, border_style);
        }
        self.state.buffer_mut().put(0, 0, bx.tl_heavy, border_style);
        self.state
            .buffer_mut()
            .put(w.saturating_sub(1), 0, bx.tr_heavy, border_style);
        self.state
            .buffer_mut()
            .put(0, h.saturating_sub(1), bx.bl_heavy, border_style);
        self.state
            .buffer_mut()
            .put(w.saturating_sub(1), h.saturating_sub(1), bx.br_heavy, border_style);

        // Title
        if !self.title_text.is_empty() {
            let title = format!(" {} ", self.title_text);
            self.state.buffer_mut().print(2, 0, &title, border_style);
        }

        // Message
        let inner_w = w.saturating_sub(4) as usize;
        let msg_y = 2u16;
        for (i, line) in self.message.lines().enumerate() {
            let y = msg_y + i as u16;
            if y >= h.saturating_sub(2) {
                break;
            }
            let visible: String = line.chars().take(inner_w).collect();
            self.state.buffer_mut().print(2, y, &visible, normal);
        }

        // Buttons at bottom
        let btn_y = h.saturating_sub(2);
        let total_btn_width: u16 = self.buttons.iter().map(|b| b.len() as u16 + 4).sum();
        let mut bx_pos = w.saturating_sub(total_btn_width) / 2;
        for (i, btn) in self.buttons.iter().enumerate() {
            let style = if i == self.focused_button {
                btn_focused
            } else {
                btn_normal
            };
            let label = format!("[ {} ]", btn);
            self.state.buffer_mut().print(bx_pos, btn_y, &label, style);
            bx_pos += label.len() as u16 + 1;
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Consumed;
        };
        match key.code {
            KeyCode::Left | KeyCode::Tab => {
                if self.focused_button > 0 {
                    self.focused_button -= 1;
                    self.state.mark_dirty();
                }
                HandleResult::Consumed
            }
            KeyCode::Right | KeyCode::BackTab => {
                if self.focused_button + 1 < self.buttons.len() {
                    self.focused_button += 1;
                    self.state.mark_dirty();
                }
                HandleResult::Consumed
            }
            KeyCode::Enter => {
                let cmd = if self.focused_button == 0 {
                    CM_OK
                } else {
                    CM_CANCEL
                };
                self.state.put_command(cmd, Some(Box::new(self.focused_button)));
                HandleResult::Consumed
            }
            KeyCode::Esc => {
                self.state.put_command(CM_CANCEL, None);
                HandleResult::Consumed
            }
            _ => HandleResult::Consumed,
        }
    }
}
