//! Buffer image placement methods.

use std::sync::Arc;

use crate::buffer::Buffer;
use crate::geometry::Rect;
use crate::image::{ImageData, ImagePlacement, ImageTransform};

impl Buffer {
    /// Blit another buffer onto this one at (dx, dy) with clipping.
    /// Transfers image placements with offset.
    pub fn blit(&mut self, src: &Buffer, dx: u16, dy: u16) {
        use crate::cell::Color;
        let src_w = src.width().min(self.width().saturating_sub(dx));
        let src_h = src.height().min(self.height().saturating_sub(dy));
        for row in 0..src_h {
            for col in 0..src_w {
                let cell = src.cell(col, row);
                if cell.style().fg() == Color::Transparent && cell.style().bg() == Color::Transparent {
                    continue;
                }
                let di = (dy + row) as usize * self.width() as usize + (dx + col) as usize;
                self.cells_mut()[di] = cell.clone();
            }
        }
        for img in src.images() {
            let r = img.rect();
            let offset_rect = Rect::new(r.x() + dx, r.y() + dy, r.w(), r.h());
            self.images.push(ImagePlacement::new(offset_rect, img.data().clone(), img.transform()));
        }
    }

    /// Place an image behind text in the given cell rect.
    pub fn place_image(&mut self, rect: Rect, data: Arc<ImageData>, transform: ImageTransform) {
        self.images.push(ImagePlacement::new(rect, data, transform));
    }

    /// Remove all image placements.
    pub fn clear_images(&mut self) {
        self.images.clear();
    }

    /// Access image placements for rendering.
    pub fn images(&self) -> &[ImagePlacement] {
        &self.images
    }
}
