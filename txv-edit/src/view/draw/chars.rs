//! Character-level line rendering with wrap, tabs, and style composition.

use txv_core::prelude::*;

use super::style::compose_char_style;
use super::DrawParams;
use super::LineDraw;
use crate::editor::Editor;
use crate::highlight::HlSpan;
use crate::view::delegate::EditorViewDelegate;

/// Draw a single buffer line's characters. Returns last visual row used.
pub fn draw_line_chars<D: EditorViewDelegate>(
    buf: &mut Buffer,
    editor: &Editor,
    delegate: &D,
    line_idx: usize,
    start_row: usize,
    p: &DrawParams,
    spans: &[HlSpan],
) -> usize {
    let mut st = LineDraw {
        col: 0,
        char_idx: 0,
        byte_pos: editor.buf().line_col_to_offset(line_idx, 0).unwrap_or(0),
        vis_row: start_row,
    };
    let mut ctx = DrawCtx {
        buf,
        editor,
        delegate,
        line_idx,
        p,
    };

    render_spans(&mut ctx, spans, &mut st);
    draw_line_tail(ctx.buf, editor, line_idx, start_row, p, &st);
    st.vis_row
}

fn render_spans<D: EditorViewDelegate>(ctx: &mut DrawCtx<'_, D>, spans: &[HlSpan], st: &mut LineDraw) {
    for span in spans {
        for ch in span.text().chars() {
            if ch == '\n' || ch == '\r' {
                st.byte_pos += ch.len_utf8();
                continue;
            }
            if ch == '\t' {
                draw_tab(ctx, span.style(), st);
            } else {
                draw_char(ctx, ch, span.style(), st);
            }
            if st.vis_row >= ctx.p.h as usize {
                return;
            }
        }
    }
}

fn draw_tab<D: EditorViewDelegate>(ctx: &mut DrawCtx<'_, D>, span_style: Style, st: &mut LineDraw) {
    let text_x = ctx.p.gutter_w;
    for ti in 0..ctx.p.tab_width {
        if st.vis_row >= ctx.p.h as usize {
            return;
        }
        if st.col >= ctx.p.h_off && (st.col - ctx.p.h_off) < ctx.p.avail {
            let x = text_x + (st.col - ctx.p.h_off) as u16;
            let style = compose_char_style(
                ctx.editor,
                ctx.delegate,
                span_style,
                ctx.line_idx,
                st.char_idx,
                st.byte_pos,
                ctx.p,
            );
            let ch = tab_display_char(ctx.editor, ti, ctx.p.tab_width);
            ctx.buf.put(x, st.vis_row as u16, ch, style);
        }
        st.col += 1;
        if ctx.p.wrap && st.col >= ctx.p.avail {
            st.vis_row += 1;
            st.col = 0;
        }
    }
    st.char_idx += 1;
    st.byte_pos += 1;
}

fn tab_display_char(editor: &Editor, ti: usize, tab_w: usize) -> char {
    if editor.options().list() {
        if ti == tab_w - 1 {
            '\u{2192}'
        } else {
            '\u{2500}'
        }
    } else {
        ' '
    }
}

/// Context passed to character draw helpers.
struct DrawCtx<'a, D> {
    buf: &'a mut Buffer,
    editor: &'a Editor,
    delegate: &'a D,
    line_idx: usize,
    p: &'a DrawParams,
}

#[allow(clippy::too_many_arguments)]
fn draw_char<D: EditorViewDelegate>(ctx: &mut DrawCtx<'_, D>, ch: char, span_style: Style, st: &mut LineDraw) {
    let text_x = ctx.p.gutter_w;
    if ctx.p.wrap && st.col >= ctx.p.avail {
        st.vis_row += 1;
        st.col = 0;
        if st.vis_row >= ctx.p.h as usize {
            return;
        }
    }
    if !ctx.p.wrap && st.col >= ctx.p.h_off + ctx.p.avail {
        st.char_idx += 1;
        st.byte_pos += ch.len_utf8();
        return;
    }
    if st.col >= ctx.p.h_off && st.vis_row < ctx.p.h as usize {
        let x = text_x + (st.col - ctx.p.h_off) as u16;
        let style = compose_char_style(
            ctx.editor,
            ctx.delegate,
            span_style,
            ctx.line_idx,
            st.char_idx,
            st.byte_pos,
            ctx.p,
        );
        let (display, style) = resolve_display(ctx.editor, ch, style);
        ctx.buf.put(x, st.vis_row as u16, display, style);
    }
    st.col += display_char_width(ch) as usize;
    st.char_idx += 1;
    st.byte_pos += ch.len_utf8();
}

fn draw_line_tail(buf: &mut Buffer, editor: &Editor, line_idx: usize, start_row: usize, p: &DrawParams, st: &LineDraw) {
    let text_x = p.gutter_w;
    if st.vis_row < p.h as usize && st.col >= p.h_off {
        let fill_start = (st.col - p.h_off).min(p.avail);
        let fill_style = ephemeral_bg(editor, line_idx);
        for fc in fill_start..p.avail {
            buf.put(text_x + fc as u16, st.vis_row as u16, ' ', fill_style);
        }
    }
    if editor.options().list() && st.vis_row < p.h as usize && st.col >= p.h_off {
        let eol_x = (st.col - p.h_off).min(p.avail);
        if eol_x < p.avail {
            let style = palette().style(StyleId::Dim);
            buf.put(text_x + eol_x as u16, st.vis_row as u16, '$', style);
        }
    }
    if editor.options().guides() {
        let line = editor.buf().line(line_idx).unwrap_or_default();
        draw_indent_guides(buf, &line, text_x, start_row as u16, p);
    }
}

fn resolve_display(editor: &Editor, ch: char, style: Style) -> (char, Style) {
    if editor.options().list() && ch == ' ' {
        let dim = palette().style(StyleId::Dim);
        ('\u{00B7}', Style::new(dim.fg(), style.bg()).with_attrs(style.attrs()))
    } else {
        (ch, style)
    }
}

fn ephemeral_bg(editor: &Editor, line_idx: usize) -> Style {
    if editor.ephemeral().ranges().iter().any(|r| r.covers_line(line_idx)) {
        Style::default().with_bg(palette().style(StyleId::SearchMatch).bg())
    } else {
        Style::default()
    }
}

fn draw_indent_guides(buf: &mut Buffer, line: &str, text_x: u16, vy: u16, p: &DrawParams) {
    let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
    let indent_visual = if line.starts_with('\t') {
        indent * p.tab_width
    } else {
        indent
    };
    let style = palette().style(StyleId::EditorGutter);
    let mut g = p.tab_width;
    while g < indent_visual && g < p.avail {
        buf.put(text_x + g as u16, vy, '\u{250A}', style);
        g += p.tab_width;
    }
}
