//! Hardware cursor shape and position request.

/// Terminal cursor shape (DECSCUSR).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorShape {
    Block,
    Underline,
    Bar,
    Hidden,
}

/// A view's request to show the hardware cursor at a position (relative to its bounds).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CursorRequest {
    pub(crate) x: u16,
    pub(crate) y: u16,
    pub(crate) shape: CursorShape,
}

impl CursorRequest {
    pub fn new(x: u16, y: u16, shape: CursorShape) -> Self {
        Self { x, y, shape }
    }

    pub fn x(&self) -> u16 {
        self.x
    }

    pub fn y(&self) -> u16 {
        self.y
    }

    pub fn shape(&self) -> CursorShape {
        self.shape
    }
}
