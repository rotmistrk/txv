//! Types for TabBar configuration.

/// Tab bar display mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabBarMode {
    /// Only active tab name shown + `▾N──` badge. Current kairn style.
    Single,
    /// All tabs visible, fixed positions, all numbered: `₁Files ₂Git ₃Tools`.
    Static,
    /// Active leftmost (no number), rest by recency: `mod.rs ₁buf.rs ▾…3`.
    Lru,
}

/// Subscript digits for tab numbering.
pub(crate) const SUBSCRIPTS: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];
