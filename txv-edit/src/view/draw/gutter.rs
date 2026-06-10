//! Gutter rendering: line numbers + delegate sign area.

use txv_core::prelude::*;

use super::DrawParams;
use crate::editor::Editor;
use crate::view::delegate::EditorViewDelegate;

pub fn draw_gutter<D: EditorViewDelegate>(
    buf: &mut Buffer,
    editor: &Editor,
    delegate: &D,
    line_idx: usize,
    y: u16,
    p: &DrawParams,
) {
    if p.gutter_w == 0 {
        return;
    }
    let extra_gw = delegate.extra_gutter_width();
    let num_w = p.gutter_w - extra_gw;

    if extra_gw > 0 {
        if let Some((ch, style)) = delegate.gutter_sign(line_idx) {
            buf.put(0, y, ch, style);
        }
    }

    if editor.options().number() && num_w > 0 {
        let style = palette().style(StyleId::EditorGutter);
        let s = format!("{:>w$} ", line_idx + 1, w = (num_w - 1) as usize);
        buf.print(extra_gw, y, &s, style);
    }
}
