//! ImagePlacement — a placed image within a Buffer.

use std::sync::Arc;

use crate::geometry::Rect;

use super::ImageData;

/// How to fit the image within its bounding cell rect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageTransform {
    /// Scale to fit within rect, preserving aspect ratio (letterbox).
    Fit,
    /// Scale to fill rect, preserving aspect ratio (crop overflow).
    Fill,
    /// Stretch to fill rect exactly (distort).
    Stretch,
}

/// A placed image in a Buffer — defines where and how to render it.
#[derive(Clone, Debug)]
pub struct ImagePlacement {
    pub(crate) rect: Rect,
    pub(crate) data: Arc<ImageData>,
    pub(crate) transform: ImageTransform,
}

impl ImagePlacement {
    pub fn new(rect: Rect, data: Arc<ImageData>, transform: ImageTransform) -> Self {
        Self { rect, data, transform }
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn data(&self) -> &Arc<ImageData> {
        &self.data
    }

    pub fn transform(&self) -> ImageTransform {
        self.transform
    }
}
