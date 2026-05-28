//! InputLine — single-line text input with history and completion.

use std::sync::Arc;

use txv_core::prelude::*;

pub struct InputLine {
    state: ViewState,
    text: String,
    cursor: usize,
    history: Vec<String>,
    history_pos: Option<usize>,
    completer: Option<Box<dyn Completer>>,
    submit_command: CommandId,
    palette: Option<Arc<dyn Palette>>,
}

impl InputLine {
    pub fn new() -> Self {
        Self {
            state: ViewState::default(),
            text: String::new(),
            cursor: 0,
            history: Vec::new(),
            history_pos: None,
            completer: None,
            submit_command: CM_OK,
            palette: None,
        }
    }

    pub fn with_command(mut self, id: CommandId) -> Self {
        self.submit_command = id;
        self
    }

    pub fn with_completer(mut self, c: Box<dyn Completer>) -> Self {
        self.completer = Some(c);
        self
    }

    pub fn set_completer(&mut self, c: Box<dyn Completer>) {
        self.completer = Some(c);
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor_pos(&self) -> usize {
        self.cursor
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.cursor = self.text.len();
        self.update_width();
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.update_width();
    }

    fn update_width(&mut self) {
        let w = (self.text.len() as u16).saturating_add(2).max(10);
        let b = self.state.bounds();
        if b.w != w {
            self.state.set_bounds(Rect::new(b.x, b.y, w, 1));
        }
        self.state.mark_dirty();
    }

    fn push_history(&mut self) {
        if !self.text.is_empty() {
            self.history.push(self.text.clone());
        }
        self.history_pos = None;
    }

    fn handle_char(&mut self, ch: char) {
        self.text.insert(self.cursor, ch);
        self.cursor += 1;
        self.update_width();
    }

    fn handle_backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.text.remove(self.cursor);
            self.update_width();
        }
    }

    fn handle_delete(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
            self.update_width();
        }
    }

    fn handle_history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let pos = match self.history_pos {
            Some(p) => p.saturating_sub(1),
            None => self.history.len() - 1,
        };
        self.history_pos = Some(pos);
        self.text = self.history[pos].clone();
        self.cursor = self.text.len();
        self.update_width();
    }

    fn handle_history_down(&mut self) {
        let Some(pos) = self.history_pos else {
            return;
        };
        if pos + 1 < self.history.len() {
            self.history_pos = Some(pos + 1);
            self.text = self.history[pos + 1].clone();
        } else {
            self.history_pos = None;
            self.text.clear();
        }
        self.cursor = self.text.len();
        self.update_width();
    }

    fn handle_command(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        if let Some(text) = data.as_ref().and_then(|d| d.downcast_ref::<String>()) {
            self.set_text(text);
            return HandleResult::Consumed;
        }
        HandleResult::Ignored
    }

    fn try_complete(&mut self) {
        let Some(ref completer) = self.completer else {
            return;
        };
        let mut first: Option<String> = None;
        let mut count = 0u32;
        let _ = completer.complete(&self.text, self.cursor, &mut |c| {
            count += 1;
            if count == 1 {
                first = Some(c.text().to_string());
            }
            Ok(count < 2)
        });
        if count == 1 {
            if let Some(text) = first {
                self.text = text;
                self.cursor = self.text.len();
                self.update_width();
            }
        }
    }

    fn resolve_style(&self, id: StyleId) -> Style {
        match &self.palette {
            Some(p) => p.style(id),
            None => txv_core::palette::palette().style(id),
        }
    }

    fn visible_start(&self, width: usize) -> usize {
        if self.cursor >= width {
            self.cursor - width + 1
        } else {
            0
        }
    }
}

impl Default for InputLine {
    fn default() -> Self {
        Self::new()
    }
}

impl View for InputLine {
    delegate_view_state!(state, override { cursor });

    fn cursor(&self) -> Option<txv_core::cursor::CursorRequest> {
        if !self.state.is_focused() {
            return None;
        }
        let w = self.state.bounds().w as usize;
        let start = self.visible_start(w);
        Some(txv_core::cursor::CursorRequest {
            x: (self.cursor - start) as u16,
            y: 0,
            shape: txv_core::cursor::CursorShape::Bar,
        })
    }

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        if w == 0 || self.state.buffer_mut().height() == 0 {
            return;
        }
        let style = self.resolve_style(StyleId::StatusBar);
        self.state.buffer_mut().hline(0, 0, w, ' ', style);
        let ww = w as usize;
        let start = self.visible_start(ww);
        let visible: String = self.text.chars().skip(start).take(ww).collect();
        self.state.buffer_mut().print(0, 0, &visible, style);
        let cx = (self.cursor - start) as u16;
        if cx < w {
            let ch = self.text.chars().nth(self.cursor).unwrap_or(' ');
            let cs = self.resolve_style(StyleId::InputCursor);
            self.state.buffer_mut().put(cx, 0, ch, cs);
        }
    }

    fn set_palette(&mut self, palette: Arc<dyn Palette>) {
        self.palette = Some(palette);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Command { data, .. } = event {
            return self.handle_command(data);
        }
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        match &key.code {
            KeyCode::Char(ch) => self.handle_char(*ch),
            KeyCode::Backspace => self.handle_backspace(),
            KeyCode::Delete => self.handle_delete(),
            KeyCode::Left if self.cursor > 0 => {
                self.cursor -= 1;
                self.state.mark_dirty()
            }
            KeyCode::Right if self.cursor < self.text.len() => {
                self.cursor += 1;
                self.state.mark_dirty()
            }
            KeyCode::Home => {
                self.cursor = 0;
                self.state.mark_dirty()
            }
            KeyCode::End => {
                self.cursor = self.text.len();
                self.state.mark_dirty()
            }
            KeyCode::Up => self.handle_history_up(),
            KeyCode::Down => self.handle_history_down(),
            KeyCode::Tab => self.try_complete(),
            KeyCode::Enter => {
                self.push_history();
                self.state
                    .put_command(self.submit_command, Some(Box::new(self.text.clone())));
            }
            KeyCode::Esc => self.state.put_command(CM_CANCEL, None),
            _ => return HandleResult::Ignored,
        }
        HandleResult::Consumed
    }
}
