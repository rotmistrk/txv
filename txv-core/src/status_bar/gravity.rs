//! Gravity — alignment direction for status bar items.

/// Item alignment on the status bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gravity {
    Left,
    Right,
}
