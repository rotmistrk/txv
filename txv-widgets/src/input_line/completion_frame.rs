//! CompletionFrame — draws a Frame border around the shared ListView popup.

use std::sync::{Arc, Mutex};

use txv_core::palette::palette;
use txv_core::prelude::*;

use super::completion_list::CompletionList;
use crate::list_view::ListView;

/// A View that draws a Frame border around a shared ListView.
/// The InputLine mutates the ListView directly; this view just renders it.
pub(crate) struct CompletionFrame {
    state: ViewState,
    list: Arc<Mutex<ListView<CompletionList>>>,
    count: usize,
}

impl CompletionFrame {
    pub fn new(list: Arc<Mutex<ListView<CompletionList>>>) -> Self {
        Self {
            state: ViewState::new(ViewOptions::default()),
            list,
            count: 0,
        }
    }

    pub fn set_count(&mut self, count: usize) {
        self.count = count;
        self.state.mark_dirty();
    }
}

impl View for CompletionFrame {
    delegate_view_state!(state, override { draw, needs_redraw });

    fn needs_redraw(&self) -> bool {
        if self.state.is_dirty() {
            return true;
        }
        self.list.lock().map(|lv| lv.needs_redraw()).unwrap_or(false)
    }

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w < 4 || h < 2 {
            return;
        }
        let style = palette().style(StyleId::Border);
        let bg = palette().style(StyleId::StatusBar);
        self.state.buffer_mut().fill(' ', bg);
        self.draw_border(w, h, style);

        // Count label on top border
        let label = format!(" {} ", self.count);
        let x = w.saturating_sub(label.len() as u16 + 1);
        self.state.buffer_mut().print(x, 0, &label, style);

        self.blit_inner_list(w, h);
    }

    fn handle(&mut self, _event: &Event) -> HandleResult {
        HandleResult::Ignored
    }
}

impl CompletionFrame {
    fn draw_border(&mut self, w: u16, h: u16, style: Style) {
        let g = glyphs();
        let bx = &g.box_drawing();
        self.state.buffer_mut().hline(0, 0, w, bx.h(), style);
        self.state.buffer_mut().hline(0, h - 1, w, bx.h(), style);
        for row in 1..h - 1 {
            self.state.buffer_mut().put(0, row, bx.v(), style);
            self.state.buffer_mut().put(w - 1, row, bx.v(), style);
        }
        self.state.buffer_mut().put(0, 0, bx.tl(), style);
        self.state.buffer_mut().put(w - 1, 0, bx.tr(), style);
        self.state.buffer_mut().put(0, h - 1, bx.bl(), style);
        self.state.buffer_mut().put(w - 1, h - 1, bx.br(), style);
    }

    fn blit_inner_list(&mut self, w: u16, h: u16) {
        let inner_w = w - 2;
        let inner_h = h - 2;
        if inner_w == 0 || inner_h == 0 {
            return;
        }
        if let Ok(mut lv) = self.list.lock() {
            lv.set_bounds(Rect::new(0, 0, inner_w, inner_h));
            lv.draw();
            self.state.buffer_mut().blit(lv.buffer(), 1, 1);
            lv.mark_redrawn();
        }
    }
}
