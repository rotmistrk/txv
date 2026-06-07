//! CursorPos — cursor position data emitted with CM_CURSOR_MOVED.

/// Cursor position data emitted with CM_CURSOR_MOVED.
#[derive(Debug, Clone, Copy)]
pub struct CursorPos {
    line: u32,
    col: u32,
}

impl CursorPos {
    pub fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn col(&self) -> u32 {
        self.col
    }
}
