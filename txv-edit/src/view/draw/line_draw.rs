//! LineDraw — mutable draw state tracking position within a line.

/// Mutable draw state tracking position within a line.
pub(crate) struct LineDraw {
    pub(crate) col: usize,
    pub(crate) char_idx: usize,
    pub(crate) byte_pos: usize,
    pub(crate) vis_row: usize,
}
