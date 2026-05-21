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
    pub x: u16,
    pub y: u16,
    pub shape: CursorShape,
}
