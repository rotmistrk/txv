//! Scrollbar — vertical scroll indicator View.

use txv_core::palette::palette;
use txv_core::prelude::*;

use crate::scroll_view::ScrollView;

pub struct Scrollbar {
    state: ViewState,
    pub(crate) scroll: ScrollView,
}

impl Scrollbar {
    pub fn new() -> Self {
        Self {
            state: ViewState::new(ViewOptions::default()),
            scroll: ScrollView::new(),
        }
    }
}

impl Default for Scrollbar {
    fn default() -> Self {
        Self::new()
    }
}

impl View for Scrollbar {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let pal = palette();
        let track_style = pal.style(StyleId::ScrollbarTrack);
        let thumb_style = pal.style(StyleId::ScrollbarThumb);
        let (thumb_pos, thumb_size) = self.scroll.thumb(h);
        for row in 0..h {
            let style = if row >= thumb_pos && row < thumb_pos + thumb_size {
                thumb_style
            } else {
                track_style
            };
            self.state
                .buffer_mut()
                .put(0, row, glyphs().ui().scrollbar_track(), style);
        }
    }

    fn handle(&mut self, _event: &Event) -> HandleResult {
        HandleResult::Ignored
    }
}
