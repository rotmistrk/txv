//! Shared drawing helpers for tree-based views.

use txv_core::prelude::*;

/// Fill empty rows below drawn content with spaces.
pub fn draw_empty_rows(buf: &mut Buffer, drawn: usize, tree_h: u16, w: u16) {
    for row in drawn..tree_h as usize {
        buf.hline(0, row as u16, w, ' ', Style::default());
    }
}

/// Draw a filter status line at the bottom of the tree area.
pub fn draw_filter_status(buf: &mut Buffer, h: u16, w: u16, filter_text: &str) {
    let y = h - 1;
    let status_style = palette().style(StyleId::Dim);
    buf.hline(0, y, w, ' ', status_style);
    let display = format!("/{}", filter_text);
    buf.print(0, y, &display, status_style);
}

/// Draw text with highlighted positions (fuzzy match).
pub fn draw_highlighted_text(
    buf: &mut Buffer,
    text: &str,
    positions: &[usize],
    x: u16,
    y: u16,
    max_x: u16,
    base_style: Style,
) {
    let hl = palette().style(StyleId::StatusHighlight);
    let hl_style = Style::new(hl.fg(), base_style.bg()).with_attrs(base_style.attrs().bold());
    for (ci, ch) in text.chars().enumerate() {
        let cx = x + ci as u16;
        if cx >= max_x {
            break;
        }
        let s = if positions.contains(&ci) {
            hl_style
        } else {
            base_style
        };
        buf.put(cx, y, ch, s);
    }
}
