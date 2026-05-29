//! Buffer diff utility.

use txv_core::buffer::Buffer;

/// Compute which cells changed between current surface and previous buffer.
/// Returns list of (x, y) positions that differ.
pub fn diff_cells(current: &Buffer, previous: &Buffer) -> Vec<(u16, u16)> {
    let mut changed = Vec::new();
    let w = current.width().min(previous.width());
    let h = current.height().min(previous.height());
    for y in 0..h {
        for x in 0..w {
            let c = current.cell(x, y);
            let p = previous.cell(x, y);
            if c.ch != p.ch || c.style != p.style {
                changed.push((x, y));
            }
        }
    }
    changed
}
