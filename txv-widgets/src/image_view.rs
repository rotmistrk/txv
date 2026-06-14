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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use txv_core::group::GroupState;

    #[test]
    fn image_survives_render() {
        let mut iv = ImageView::new();
        let pixels = vec![255u8; 10 * 10 * 4];
        iv.set_image(Arc::new(ImageData::new(10, 10, pixels)));
        iv.set_bounds(Rect::new(0, 0, 20, 10));
        iv.render();
        assert_eq!(iv.buffer().images().len(), 1, "image in leaf buffer");
    }

    #[test]
    fn image_survives_parent_blit() {
        // Simulate what GroupState render does: parent buffer + blit child
        let mut iv = ImageView::new();
        let pixels = vec![255u8; 4 * 4 * 4];
        iv.set_image(Arc::new(ImageData::new(4, 4, pixels)));
        iv.set_bounds(Rect::new(0, 0, 10, 5));
        iv.render();

        // Parent buffer
        let mut parent_buf = Buffer::new(40, 20);
        parent_buf.fill(' ', Style::default());
        // Blit child at offset (5, 3)
        parent_buf.blit(iv.buffer(), 5, 3);

        assert_eq!(parent_buf.images().len(), 1, "image transferred to parent");
        let img = &parent_buf.images()[0];
        assert_eq!(img.rect().x(), 5, "x offset applied");
        assert_eq!(img.rect().y(), 3, "y offset applied");
    }
}
