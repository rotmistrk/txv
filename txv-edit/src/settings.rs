//! Editor settings types.

/// Cursor style: software (reverse block) or hardware (bar/block/underline).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Software,
    Bar,
    Block,
    Underline,
}
