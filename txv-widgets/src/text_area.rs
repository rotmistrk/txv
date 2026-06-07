//! TextArea — read-only text viewer with line numbers and search.

use txv_core::palette::palette;
use txv_core::prelude::*;

use crate::scroll_view::ScrollView;

pub struct TextArea {
    pub(crate) state: ViewState,
    pub(crate) lines: Vec<String>,
    pub(crate) scroll: ScrollView,
    pub(crate) line_numbers_enabled: bool,
    pub(crate) search_query: String,
    pub(crate) search_matches: Vec<usize>,
    pub(crate) current_match: usize,
    /// Per-line foreground colors (optional, indexed by line number).
    pub(crate) line_colors: Vec<Color>,
    searching: bool,
    search_input: String,
}

impl TextArea {
    pub fn new() -> Self {
        Self {
            state: ViewState::default(),
            lines: Vec::new(),
            scroll: ScrollView::new(),
            line_numbers_enabled: true,
            search_query: String::new(),
            search_matches: Vec::new(),
            current_match: 0,
            line_colors: Vec::new(),
            searching: false,
            search_input: String::new(),
        }
    }

    pub fn set_content(&mut self, text: &str) {
        self.lines = text.lines().map(String::from).collect();
        self.scroll.set_total(self.lines.len());
        self.state.mark_dirty();
    }

    pub fn content(&self) -> String {
        self.lines.join("\n")
    }

    pub fn show_line_numbers(&mut self, show: bool) {
        self.line_numbers_enabled = show;
        self.state.mark_dirty();
    }

    pub fn append_lines(&mut self, text: &str) {
        for line in text.lines() {
            self.lines.push(line.to_string());
        }
        self.scroll.set_total(self.lines.len());
        self.state.mark_dirty();
    }

    fn gutter_width(&self) -> u16 {
        if !self.line_numbers_enabled {
            return 0;
        }
        let digits = if self.lines.is_empty() {
            1
        } else {
            (self.lines.len() as f64).log10() as u16 + 1
        };
        digits + 1 // +1 for separator space
    }
}

impl Default for TextArea {
    fn default() -> Self {
        Self::new()
    }
}

impl View for TextArea {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let gutter_w = self.gutter_width();
        let content_h = if self.searching {
            h.saturating_sub(1) as usize
        } else {
            h as usize
        };
        self.draw_content_lines(w, gutter_w, content_h);
        if self.searching {
            self.draw_search_prompt(w, h);
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        if self.searching {
            return self.handle_search_input(key);
        }
        self.handle_navigation(key)
    }
}

impl TextArea {
    fn draw_content_lines(&mut self, w: u16, gutter_w: u16, content_h: usize) {
        let pal = palette();
        let gutter_style = pal.style(StyleId::Dim);
        let normal = Style::default();
        let highlight = pal.style(StyleId::SearchMatch);

        for row in 0..content_h {
            let line_idx = self.scroll.offset + row;
            let y = row as u16;
            self.state.buffer_mut().hline(0, y, w, ' ', normal);
            if line_idx >= self.lines.len() {
                continue;
            }
            if self.line_numbers_enabled {
                let num = format!("{:>width$} ", line_idx + 1, width = (gutter_w - 1) as usize);
                self.state.buffer_mut().print(0, y, &num, gutter_style);
            }
            let is_match = self.search_matches.contains(&line_idx);
            let style = if is_match {
                highlight
            } else if let Some(&color) = self.line_colors.get(line_idx) {
                Style::default().with_fg(color)
            } else {
                normal
            };
            let avail = w.saturating_sub(gutter_w) as usize;
            let visible: String = self.lines[line_idx].chars().take(avail).collect();
            self.state.buffer_mut().print(gutter_w, y, &visible, style);
        }
    }

    fn draw_search_prompt(&mut self, w: u16, h: u16) {
        let y = h.saturating_sub(1);
        let prompt_style = palette().style(StyleId::StatusBar);
        self.state.buffer_mut().hline(0, y, w, ' ', prompt_style);
        let prompt = format!("/{}", self.search_input);
        self.state.buffer_mut().print(0, y, &prompt, prompt_style);
    }

    fn handle_search_input(&mut self, key: &txv_core::event::KeyEvent) -> HandleResult {
        match key.code() {
            KeyCode::Enter => {
                self.searching = false;
                self.search(&self.search_input.clone());
            }
            KeyCode::Esc => {
                self.searching = false;
                self.search_input.clear();
                self.state.mark_dirty();
            }
            KeyCode::Backspace => {
                self.search_input.pop();
                self.state.mark_dirty();
            }
            KeyCode::Char(ch) => {
                self.search_input.push(ch);
                self.state.mark_dirty();
            }
            _ => {}
        }
        HandleResult::Consumed
    }

    fn handle_navigation(&mut self, key: &txv_core::event::KeyEvent) -> HandleResult {
        match key.code() {
            KeyCode::Up => self.scroll_and_dirty(|s| s.scroll.scroll_up(1)),
            KeyCode::Down => self.scroll_and_dirty(|s| s.scroll.scroll_down(1)),
            KeyCode::PageUp => {
                let page = (self.state.bounds().h() as usize).saturating_sub(1).max(1);
                self.scroll_and_dirty(|s| s.scroll.scroll_up(page))
            }
            KeyCode::PageDown => {
                let page = (self.state.bounds().h() as usize).saturating_sub(1).max(1);
                self.scroll_and_dirty(|s| s.scroll.scroll_down(page))
            }
            KeyCode::Home => self.scroll_and_dirty(|s| s.scroll.scroll_to(0)),
            KeyCode::End => {
                let max = self.scroll.max_offset();
                self.scroll_and_dirty(|s| s.scroll.scroll_to(max))
            }
            KeyCode::Char('/') if !key.modifiers().ctrl() => {
                self.searching = true;
                self.search_input.clear();
                self.state.mark_dirty();
                HandleResult::Consumed
            }
            KeyCode::Char('n') if !key.modifiers().ctrl() => {
                self.next_match();
                HandleResult::Consumed
            }
            KeyCode::Char('N') if !key.modifiers().ctrl() => {
                self.prev_match();
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }

    fn scroll_and_dirty(&mut self, f: impl FnOnce(&mut Self)) -> HandleResult {
        f(self);
        self.state.mark_dirty();
        HandleResult::Consumed
    }
}
