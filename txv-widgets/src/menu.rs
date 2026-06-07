//! Menu — modal popup menu.

use txv_core::palette::palette;
use txv_core::prelude::*;

use crate::menu_item::MenuItem;

pub struct Menu {
    state: ViewState,
    pub(crate) items: Vec<MenuItem>,
    pub(crate) cursor: usize,
}

impl Menu {
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            state: ViewState::new(ViewOptions::default().with_focusable().with_modal()),
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
        let pal = palette();
        let normal = Style::default();
        let selected = pal.style(StyleId::CursorFocused);
        let disabled = pal.style(StyleId::Disabled);
        self.draw_menu_border(w, h, normal);
        self.draw_menu_items(w, h, normal, selected, disabled);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Consumed;
        };
        match key.code() {
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
            _ => HandleResult::Consumed,
        }
    }
}

impl Menu {
    fn draw_menu_border(&mut self, w: u16, h: u16, normal: Style) {
        let g = glyphs();
        let bx = &g.box_drawing();
        self.state.buffer_mut().hline(0, 0, w, bx.h(), normal);
        self.state.buffer_mut().hline(0, h.saturating_sub(1), w, bx.h(), normal);
        for row in 1..h.saturating_sub(1) {
            self.state.buffer_mut().put(0, row, bx.v(), normal);
            self.state.buffer_mut().put(w.saturating_sub(1), row, bx.v(), normal);
        }
        self.state.buffer_mut().put(0, 0, bx.tl(), normal);
        self.state.buffer_mut().put(w.saturating_sub(1), 0, bx.tr(), normal);
        self.state.buffer_mut().put(0, h.saturating_sub(1), bx.bl(), normal);
        self.state
            .buffer_mut()
            .put(w.saturating_sub(1), h.saturating_sub(1), bx.br(), normal);
    }

    fn draw_menu_items(&mut self, w: u16, h: u16, normal: Style, selected: Style, disabled: Style) {
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
}
