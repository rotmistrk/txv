//! ListView — generic list widget parameterized by ListData.

use txv_core::prelude::*;

use crate::scroll_view::ScrollView;

/// Trait for providing list data to ListView.
pub trait ListData: Send + 'static {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn label(&self, index: usize) -> &str;
    fn style(&self, index: usize) -> Style;
}

pub struct ListView<D: ListData> {
    state: ViewState,
    data: D,
    cursor: usize,
    scroll: ScrollView,
}

impl<D: ListData> ListView<D> {
    pub fn new(data: D) -> Self {
        Self {
            state: ViewState::default(),
            data,
            cursor: 0,
            scroll: ScrollView::new(),
        }
    }

    pub fn data(&self) -> &D {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut D {
        self.state.mark_dirty();
        &mut self.data
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_cursor(&mut self, pos: usize) {
        self.cursor = pos.min(self.data.len().saturating_sub(1));
        self.sync_scroll();
        self.state.mark_dirty();
    }

    pub fn select_next(&mut self) {
        let max = self.data.len().saturating_sub(1);
        if self.cursor < max {
            self.cursor += 1;
            self.sync_scroll();
            self.state.mark_dirty();
        }
    }

    pub fn select_prev(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.sync_scroll();
            self.state.mark_dirty();
        }
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
        self.scroll.scroll_to(0);
        self.state.mark_dirty();
    }

    fn sync_scroll(&mut self) {
        let h = self.state.bounds().h as usize;
        self.scroll.set_viewport(h);
        self.scroll.set_total(self.data.len());
        self.scroll.ensure_visible(self.cursor);
    }
}

impl<D: ListData> View for ListView<D> {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        self.sync_scroll();
        let selected = if self.state.is_focused() {
            txv_core::palette::palette().style(StyleId::CursorFocused)
        } else {
            txv_core::palette::palette().style(StyleId::PopupSelected)
        };
        for row in 0..h as usize {
            let idx = self.scroll.offset + row;
            if idx >= self.data.len() {
                break;
            }
            let style = if idx == self.cursor {
                selected
            } else {
                self.data.style(idx)
            };
            let y = row as u16;
            self.state.buffer_mut().hline(0, y, w, ' ', style);
            self.state.buffer_mut().print(1, y, self.data.label(idx), style);
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        match key.code {
            KeyCode::Up => {
                self.select_prev();
                HandleResult::Consumed
            }
            KeyCode::Down => {
                self.select_next();
                HandleResult::Consumed
            }
            KeyCode::Home => {
                self.set_cursor(0);
                HandleResult::Consumed
            }
            KeyCode::End => {
                let last = self.data.len().saturating_sub(1);
                self.set_cursor(last);
                HandleResult::Consumed
            }
            KeyCode::PageDown => {
                let page = (self.state.bounds().h as usize).saturating_sub(1).max(1);
                let max = self.data.len().saturating_sub(1);
                self.set_cursor((self.cursor + page).min(max));
                HandleResult::Consumed
            }
            KeyCode::PageUp => {
                let page = (self.state.bounds().h as usize).saturating_sub(1).max(1);
                self.set_cursor(self.cursor.saturating_sub(page));
                HandleResult::Consumed
            }
            KeyCode::Enter => {
                self.state.put_command(CM_OK, Some(Box::new(self.cursor)));
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}
