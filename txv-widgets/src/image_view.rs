//! ImageView — a leaf widget that displays an image filling its bounds.

use std::sync::Arc;

use txv_core::prelude::*;

/// A view that renders an image behind transparent cells.
pub struct ImageView {
    state: ViewState,
    data: Option<Arc<ImageData>>,
    transform: ImageTransform,
}

impl Default for ImageView {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageView {
    pub fn new() -> Self {
        Self {
            state: ViewState::new(ViewOptions::default()),
            data: None,
            transform: ImageTransform::Fit,
        }
    }

    pub fn set_image(&mut self, data: Arc<ImageData>) {
        self.data = Some(data);
        self.state.mark_dirty();
    }

    pub fn set_transform(&mut self, transform: ImageTransform) {
        self.transform = transform;
        self.state.mark_dirty();
    }
}

impl View for ImageView {
    delegate_view_state!(state, override { draw, handle });

    fn draw(&mut self) {
        let b = self.state.bounds();
        // Fill with transparent so image shows through
        let transparent = Style::new(Color::Transparent, Color::Transparent);
        self.state.buffer_mut().fill(' ', transparent);
        // Place image
        if let Some(ref data) = self.data {
            let rect = Rect::new(0, 0, b.w(), b.h());
            self.state.buffer_mut().place_image(rect, data.clone(), self.transform);
        }
    }

    fn handle(&mut self, _event: &Event) -> HandleResult {
        HandleResult::Ignored
    }
}
