//! InputDialog — modal dialog with a single-line text input.
//! Emits CM_OK with the entered text, or CM_CANCEL on Esc.

use txv_core::prelude::*;

/// A modal prompt dialog with a title and single-line input.
pub struct InputDialog {
    state: ViewState,
    title_text: String,
    text: String,
    cursor: usize,
}

impl InputDialog {
    pub fn new(title: impl Into<String>) -> Self {
        let mut s = Self {
            state: ViewState::new(ViewOptions {
                modal: true,
                focusable: true,
                ..ViewOptions::default()
            }),
            title_text: title.into(),
            text: String::new(),
            cursor: 0,
        };
        s.state.set_title(s.title_text.clone());
        s
    }

    /// Get the entered text.
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl View for InputDialog {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let normal = Style::default();
        let border = txv_core::palette::palette().base.border.to_style();
        // Fill background
        for row in 0..h {
            self.state.buffer_mut().hline(0, row, w, ' ', normal);
        }
        // Border
        let g = glyphs();
        let bx = &g.box_drawing;
        self.state.buffer_mut().hline(0, 0, w, bx.h_heavy, border);
        self.state
            .buffer_mut()
            .hline(0, h.saturating_sub(1), w, bx.h_heavy, border);
        for row in 1..h.saturating_sub(1) {
            self.state.buffer_mut().put(0, row, bx.v_heavy, border);
            self.state
                .buffer_mut()
                .put(w.saturating_sub(1), row, bx.v_heavy, border);
        }
        self.state.buffer_mut().put(0, 0, bx.tl_heavy, border);
        self.state.buffer_mut().put(w.saturating_sub(1), 0, bx.tr_heavy, border);
        self.state.buffer_mut().put(0, h.saturating_sub(1), bx.bl_heavy, border);
        self.state
            .buffer_mut()
            .put(w.saturating_sub(1), h.saturating_sub(1), bx.br_heavy, border);
        // Title
        if !self.title_text.is_empty() {
            let title = format!(" {} ", self.title_text);
            self.state.buffer_mut().print(2, 0, &title, border);
        }
        // Input line
        let inner_w = w.saturating_sub(4) as usize;
        let input_y = 2u16;
        let start = if self.cursor >= inner_w {
            self.cursor - inner_w + 1
        } else {
            0
        };
        let visible: String = self.text.chars().skip(start).take(inner_w).collect();
        self.state.buffer_mut().print(2, input_y, &visible, normal);
        // Cursor
        let cx = (self.cursor - start) as u16;
        if cx < inner_w as u16 {
            let ch = self.text.chars().nth(self.cursor).unwrap_or(' ');
            let cursor_style = txv_core::palette::palette().interactive.input_cursor.to_style();
            self.state.buffer_mut().put(2 + cx, input_y, ch, cursor_style);
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Consumed;
        };
        match &key.code {
            KeyCode::Enter => {
                self.state.put_command(CM_OK, Some(Box::new(self.text.clone())));
            }
            KeyCode::Esc => {
                self.state.put_command(CM_CANCEL, None);
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.text.remove(self.cursor);
                    self.state.mark_dirty();
                }
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.state.mark_dirty();
                }
            }
            KeyCode::Right => {
                if self.cursor < self.text.len() {
                    self.cursor += 1;
                    self.state.mark_dirty();
                }
            }
            KeyCode::Char(ch) => {
                self.text.insert(self.cursor, *ch);
                self.cursor += 1;
                self.state.mark_dirty();
            }
            _ => {}
        }
        HandleResult::Consumed
    }
}
