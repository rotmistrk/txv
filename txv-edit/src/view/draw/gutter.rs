//! Gutter rendering: line numbers + sign column.

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
    let style = palette().style(StyleId::EditorGutter);
    for x in 0..p.gutter_w {
        buf.put(x, y, ' ', style);
    }
    if editor.options().number() {
        draw_line_number(buf, line_idx, y, num_w, extra_gw, style);
    }
    if extra_gw > 0 {
        if let Some((ch, sign_style)) = delegate.gutter_sign(line_idx) {
            buf.put(p.gutter_w - 1, y, ch, sign_style);
        }
    }
}

fn draw_line_number(buf: &mut Buffer, line_idx: usize, y: u16, num_w: u16, extra_gw: u16, style: Style) {
    if extra_gw > 0 {
        let s = format!("{:>w$}", line_idx + 1, w = num_w as usize);
        buf.print(0, y, &s, style);
    } else if num_w > 1 {
        let s = format!("{:>w$} ", line_idx + 1, w = (num_w - 1) as usize);
        buf.print(0, y, &s, style);
    }
}
