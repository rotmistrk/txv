//! Table — columnar data display with row selection.

use txv_core::prelude::*;

use crate::scroll_view::ScrollView;

pub struct Column {
    pub title: String,
    pub width: u16,
}

pub struct Table {
    state: ViewState,
    pub columns: Vec<Column>,
    pub rows: Vec<Vec<String>>,
    pub cursor: usize,
    pub scroll: ScrollView,
}

impl Table {
    pub fn new(columns: Vec<Column>) -> Self {
        Self {
            state: ViewState::default(),
            columns,
            rows: Vec::new(),
            cursor: 0,
            scroll: ScrollView::new(),
        }
    }

    pub fn set_rows(&mut self, rows: Vec<Vec<String>>) {
        self.rows = rows;
        self.scroll.set_total(self.rows.len());
        self.cursor = 0;
        self.state.mark_dirty();
    }

    fn sync_scroll(&mut self) {
        let h = self.state.bounds().h.saturating_sub(1) as usize; // -1 for header
        self.scroll.set_viewport(h);
        self.scroll.set_total(self.rows.len());
        self.scroll.ensure_visible(self.cursor);
    }
}

impl View for Table {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let pal = txv_core::palette::palette();
        let header_style = pal.style(StyleId::PopupTableHeader);
        let normal = Style::default();
        let selected = if self.state.is_focused() {
            pal.style(StyleId::CursorFocused)
        } else {
            pal.style(StyleId::CursorUnfocused)
        };

        // Header row
        self.state.buffer_mut().hline(0, 0, w, ' ', header_style);
        let mut x = 0u16;
        for col in &self.columns {
            if x >= w {
                break;
            }
            let cw = col.width.min(w - x);
            let title: String = col.title.chars().take(cw as usize).collect();
            self.state.buffer_mut().print(x, 0, &title, header_style);
            x += col.width;
        }

        // Data rows
        let data_h = h.saturating_sub(1) as usize;
        for row in 0..data_h {
            let idx = self.scroll.offset + row;
            let y = 1 + row as u16;
            if idx >= self.rows.len() {
                self.state.buffer_mut().hline(0, y, w, ' ', normal);
                continue;
            }
            let style = if idx == self.cursor {
                selected
            } else {
                normal
            };
            self.state.buffer_mut().hline(0, y, w, ' ', style);
            let mut cx = 0u16;
            for (ci, col) in self.columns.iter().enumerate() {
                if cx >= w {
                    break;
                }
                let text = self.rows[idx].get(ci).map(|s| s.as_str()).unwrap_or("");
                let cw = col.width.min(w - cx) as usize;
                let visible: String = text.chars().take(cw).collect();
                self.state.buffer_mut().print(cx, y, &visible, style);
                cx += col.width;
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        match key.code {
            KeyCode::Up => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.sync_scroll();
                    self.state.mark_dirty();
                }
                HandleResult::Consumed
            }
            KeyCode::Down => {
                let max = self.rows.len().saturating_sub(1);
                if self.cursor < max {
                    self.cursor += 1;
                    self.sync_scroll();
                    self.state.mark_dirty();
                }
                HandleResult::Consumed
            }
            KeyCode::Enter => {
                self.state.put_command(CM_OK, Some(Box::new(self.cursor)));
                HandleResult::Consumed
            }
            KeyCode::PageDown => {
                let page = self.state.bounds().h.saturating_sub(2) as usize;
                let max = self.rows.len().saturating_sub(1);
                self.cursor = (self.cursor + page).min(max);
                self.sync_scroll();
                self.state.mark_dirty();
                HandleResult::Consumed
            }
            KeyCode::PageUp => {
                let page = self.state.bounds().h.saturating_sub(2) as usize;
                self.cursor = self.cursor.saturating_sub(page);
                self.sync_scroll();
                self.state.mark_dirty();
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}
