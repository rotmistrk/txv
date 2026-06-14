//! ImageData — decoded RGBA pixel buffer.

/// Decoded RGBA image data.
#[derive(Clone, Debug)]
pub struct ImageData {
    width: u32,
    height: u32,
    pixels: Vec<u8>, // RGBA, row-major, 4 bytes per pixel
}

impl ImageData {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        debug_assert_eq!(pixels.len(), (width * height * 4) as usize);
        Self { width, height, pixels }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}
