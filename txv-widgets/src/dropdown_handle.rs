//! Event handling for DropdownMenu.

use txv_core::prelude::*;

use super::dropdown_menu::{
    DropdownMenu, FilterMode, NumberMode, CM_DROPDOWN_CANCELLED, CM_DROPDOWN_CHANGED, CM_DROPDOWN_DONE,
};
use super::dropdown_source::DropdownSource;

impl<D: DropdownSource> View for DropdownMenu<D> {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w < 3 || h < 2 {
            return;
        }
        self.sync_scroll();
        let pal = palette();
        let bg = pal.style(StyleId::PopupBackground);
        let border_style = self.border_style.unwrap_or_else(|| pal.style(StyleId::Border));
        let selected = pal.style(StyleId::CursorFocused);
        let dim = pal.style(StyleId::Dim);
        self.state.buffer_mut().fill(' ', bg);
        self.draw_frame(w, h, border_style);
        self.draw_items(w, bg, selected, dim);
        self.draw_filter_label(w, h, border_style);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        if key.modifiers().alt() && !key.modifiers().ctrl() {
            if let KeyCode::Char(ch) = key.code() {
                if ch.is_ascii_digit() && ch != '0' {
                    return self.select_by_number(ch);
                }
            }
            return HandleResult::Ignored;
        }
        if key.modifiers().ctrl() {
            if let KeyCode::Char('f') = key.code() {
                return self.cycle_filter_mode();
            }
            return HandleResult::Ignored;
        }
        match key.code() {
            KeyCode::Up => self.move_cursor(-1),
            KeyCode::Down => self.move_cursor(1),
            KeyCode::Enter => self.select_current(),
            KeyCode::Esc => {
                self.state.put_command(CM_DROPDOWN_CANCELLED, None);
                HandleResult::Consumed
            }
            KeyCode::Tab | KeyCode::Right if self.filter_enabled => self.autocomplete_lcp(),
            KeyCode::Backspace if self.filter_enabled => self.handle_backspace(),
            KeyCode::Char(ch) => self.handle_char(ch),
            _ => HandleResult::Ignored,
        }
    }
}

impl<D: DropdownSource> DropdownMenu<D> {
    pub(crate) fn move_cursor(&mut self, delta: i32) -> HandleResult {
        let len = self.visible.len();
        if len == 0 {
            return HandleResult::Consumed;
        }
        let new = if delta < 0 {
            self.cursor.saturating_sub((-delta) as usize)
        } else {
            (self.cursor + delta as usize).min(len - 1)
        };
        if new != self.cursor {
            self.cursor = new;
            self.state.mark_dirty();
            self.state.put_command(CM_DROPDOWN_CHANGED, Some(Box::new(new)));
        }
        HandleResult::Consumed
    }

    fn select_current(&mut self) -> HandleResult {
        if self.visible.is_empty() {
            return HandleResult::Consumed;
        }
        let orig = self.visible[self.cursor];
        self.state.put_command(CM_DROPDOWN_DONE, Some(Box::new(orig)));
        HandleResult::Consumed
    }

    fn handle_char(&mut self, ch: char) -> HandleResult {
        if self.filter_mode == FilterMode::None && ch.is_ascii_digit() && ch != '0' {
            return self.select_by_number(ch);
        }
        if self.filter_enabled && !ch.is_control() {
            self.filter.push(ch);
            self.refilter();
            self.cursor = 0;
            self.state.mark_dirty();
            return HandleResult::Consumed;
        }
        HandleResult::Ignored
    }

    pub(crate) fn select_by_number(&mut self, ch: char) -> HandleResult {
        let n = (ch as usize) - ('1' as usize);
        let effective = match self.number_mode {
            NumberMode::SkipFirst => n + 1,
            _ => n,
        };
        if effective < self.visible.len() {
            let orig = self.visible[effective];
            self.state.put_command(CM_DROPDOWN_DONE, Some(Box::new(orig)));
        }
        HandleResult::Consumed
    }

    fn cycle_filter_mode(&mut self) -> HandleResult {
        self.filter_mode = match self.filter_mode {
            FilterMode::None => FilterMode::None,
            FilterMode::Prefix => FilterMode::Substring,
            FilterMode::Substring => FilterMode::Subsequence,
            FilterMode::Subsequence => FilterMode::Prefix,
        };
        if !self.filter.is_empty() {
            self.refilter();
            self.cursor = 0;
        }
        self.state.mark_dirty();
        HandleResult::Consumed
    }

    fn handle_backspace(&mut self) -> HandleResult {
        if self.filter.pop().is_some() {
            self.refilter();
            self.cursor = 0;
            self.state.mark_dirty();
        }
        HandleResult::Consumed
    }

    fn autocomplete_lcp(&mut self) -> HandleResult {
        if !self.filter_enabled || self.visible.is_empty() {
            return HandleResult::Consumed;
        }
        let first = self.source.label(self.visible[0]).to_string();
        let mut lcp_len = first.len();
        for &idx in &self.visible[1..] {
            let label = self.source.label(idx);
            lcp_len = first
                .chars()
                .zip(label.chars())
                .take(lcp_len)
                .take_while(|(a, b)| a.to_lowercase().eq(b.to_lowercase()))
                .count();
            if lcp_len == 0 {
                break;
            }
        }
        let lcp: String = first.chars().take(lcp_len).collect();
        if lcp.len() > self.filter.len() {
            self.filter = lcp.to_lowercase();
            self.refilter();
            self.cursor = 0;
            self.state.mark_dirty();
        }
        HandleResult::Consumed
    }
}
