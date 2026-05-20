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
    pub data: D,
    pub cursor: usize,
    pub scroll: ScrollView,
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
        let selected = if self.state.is_focused() {
            txv_core::palette::palette().interactive.cursor_focused.to_style()
        } else {
            txv_core::palette::palette().interactive.cursor_unfocused.to_style()
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
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => {
                    if self.cursor > 0 {
                        self.cursor -= 1;
                        self.sync_scroll();
                        self.state.mark_dirty();
                    }
                    HandleResult::Consumed
                }
                KeyCode::Down => {
                    let max = self.data.len().saturating_sub(1);
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
                KeyCode::Home => {
                    self.cursor = 0;
                    self.sync_scroll();
                    self.state.mark_dirty();
                    HandleResult::Consumed
                }
                KeyCode::End => {
                    self.cursor = self.data.len().saturating_sub(1);
                    self.sync_scroll();
                    self.state.mark_dirty();
                    HandleResult::Consumed
                }
                KeyCode::PageDown => {
                    let page = (self.state.bounds().h as usize).saturating_sub(1).max(1);
                    let max = self.data.len().saturating_sub(1);
                    self.cursor = (self.cursor + page).min(max);
                    self.sync_scroll();
                    self.state.mark_dirty();
                    HandleResult::Consumed
                }
                KeyCode::PageUp => {
                    let page = (self.state.bounds().h as usize).saturating_sub(1).max(1);
                    self.cursor = self.cursor.saturating_sub(page);
                    self.sync_scroll();
                    self.state.mark_dirty();
                    HandleResult::Consumed
                }
                _ => HandleResult::Ignored,
            },
            _ => HandleResult::Ignored,
        }
    }
}
