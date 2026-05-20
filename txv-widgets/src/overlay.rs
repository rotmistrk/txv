//! Overlay — positioned popup container that wraps a child View.

use txv_core::prelude::*;

pub struct Overlay {
    state: ViewState,
    pub child: Box<dyn View>,
    pub anchor_x: u16,
    pub anchor_y: u16,
    pub width: u16,
    pub height: u16,
}

impl Overlay {
    pub fn new(child: Box<dyn View>, anchor_x: u16, anchor_y: u16, width: u16, height: u16) -> Self {
        Self {
            state: ViewState::new(ViewOptions {
                modal: true,
                focusable: true,
                ..ViewOptions::default()
            }),
            child,
            anchor_x,
            anchor_y,
            width,
            height,
        }
    }

    /// Reposition the overlay and update child bounds.
    pub fn reposition(&mut self, x: u16, y: u16, w: u16, h: u16) {
        self.anchor_x = x;
        self.anchor_y = y;
        self.width = w;
        self.height = h;
        self.child.set_bounds(Rect::new(x, y, w, h));
        self.state.mark_dirty();
    }
}

impl View for Overlay {
    delegate_view_state!(state, override { draw, set_sink });

    fn draw(&mut self) {
        self.child.draw();
        let my_bounds = self.state.bounds();
        let cb = self.child.bounds();
        let dx = cb.x.saturating_sub(my_bounds.x);
        let dy = cb.y.saturating_sub(my_bounds.y);
        self.state.buffer_mut().blit(self.child.buffer(), dx, dy);
    }

    fn set_sink(&mut self, sink: EventSink) {
        self.state.set_sink(sink.clone());
        self.child.set_sink(sink);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        self.child.handle(event)
    }
}
