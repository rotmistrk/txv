//! FuzzySelect — input line + filtered list for fuzzy file search.

use txv_core::palette::palette;
use txv_core::prelude::*;

use crate::scroll_view::ScrollView;

pub struct FuzzySelect {
    state: ViewState,
    pub(crate) query: String,
    pub(crate) cursor_pos: usize,
    pub(crate) items: Vec<String>,
    pub(crate) filtered: Vec<usize>,
    pub(crate) selected: usize,
    pub(crate) scroll: ScrollView,
}

impl FuzzySelect {
    pub fn new(items: Vec<String>) -> Self {
        let filtered: Vec<usize> = (0..items.len()).collect();
        let mut s = Self {
            state: ViewState::new(ViewOptions::default().with_focusable().with_modal()),
            query: String::new(),
            cursor_pos: 0,
            items,
            filtered,
            selected: 0,
            scroll: ScrollView::new(),
        };
        s.scroll.set_total(s.filtered.len());
        s
    }

    pub fn set_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.refilter();
    }

    fn refilter(&mut self) {
        self.filtered.clear();
        let q = self.query.to_lowercase();
        for (i, item) in self.items.iter().enumerate() {
            if q.is_empty() || fuzzy_match(&item.to_lowercase(), &q) {
                self.filtered.push(i);
            }
        }
        self.selected = 0;
        self.scroll.set_total(self.filtered.len());
        self.scroll.scroll_to(0);
        self.state.mark_dirty();
    }

    fn sync_scroll(&mut self) {
        let h = self.state.bounds().h().saturating_sub(1) as usize; // -1 for input line
        self.scroll.set_viewport(h);
        self.scroll.ensure_visible(self.selected);
    }
}

/// Simple subsequence fuzzy match.
fn fuzzy_match(haystack: &str, needle: &str) -> bool {
    let mut chars = needle.chars();
    let mut current = chars.next();
    for h in haystack.chars() {
        if let Some(c) = current {
            if h == c {
                current = chars.next();
            }
        } else {
            break;
        }
    }
    current.is_none()
}

impl View for FuzzySelect {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let pal = palette();
        let normal = Style::default();
        let selected_style = pal.style(StyleId::CursorFocused);
        let input_style = Style::default().with_attrs(Attrs::default().underline());
        self.draw_input_line(w, input_style);
        self.draw_filtered_list(w, h, normal, selected_style);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Consumed;
        };
        match key.code() {
            KeyCode::Char(ch) => {
                self.query.insert(self.cursor_pos, ch);
                self.cursor_pos += 1;
                self.refilter();
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.query.remove(self.cursor_pos);
                    self.refilter();
                }
            }
            KeyCode::Up => self.select_prev(),
            KeyCode::Down => self.select_next(),
            KeyCode::Enter => {
                if let Some(&item_idx) = self.filtered.get(self.selected) {
                    self.state.put_command(CM_OK, Some(Box::new(item_idx)));
                }
            }
            KeyCode::Esc => self.state.put_command(CM_CANCEL, None),
            _ => {}
        }
        HandleResult::Consumed
    }
}

impl FuzzySelect {
    fn select_prev(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.sync_scroll();
            self.state.mark_dirty();
        }
    }

    fn select_next(&mut self) {
        if self.selected + 1 < self.filtered.len() {
            self.selected += 1;
            self.sync_scroll();
            self.state.mark_dirty();
        }
    }
    fn draw_input_line(&mut self, w: u16, input_style: Style) {
        self.state.buffer_mut().hline(0, 0, w, ' ', input_style);
        self.state.buffer_mut().print(0, 0, "> ", input_style);
        let avail = w.saturating_sub(2) as usize;
        let visible: String = self.query.chars().take(avail).collect();
        self.state.buffer_mut().print(2, 0, &visible, input_style);
    }

    fn draw_filtered_list(&mut self, w: u16, h: u16, normal: Style, selected_style: Style) {
        let list_h = h.saturating_sub(1) as usize;
        for row in 0..list_h {
            let idx = self.scroll.offset + row;
            let y = 1 + row as u16;
            if idx >= self.filtered.len() {
                self.state.buffer_mut().hline(0, y, w, ' ', normal);
                continue;
            }
            let style = if idx == self.selected {
                selected_style
            } else {
                normal
            };
            self.state.buffer_mut().hline(0, y, w, ' ', style);
            let item_idx = self.filtered[idx];
            let text: String = self.items[item_idx].chars().take(w as usize).collect();
            self.state.buffer_mut().print(1, y, &text, style);
        }
    }
}
