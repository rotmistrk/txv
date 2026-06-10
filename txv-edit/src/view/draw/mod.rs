//! draw() implementation for EditorView — full-featured rendering.

pub mod chars;
pub mod gutter;
mod line_draw;
mod prompt;
pub mod rainbow;
pub mod sticky;
pub mod style;

use txv_core::prelude::*;

use super::EditorView;
use crate::editor::motions::match_bracket;
use crate::editor::Editor;
use crate::highlight::{HighlightCache, Highlighter, HlSpan};
use crate::view::delegate::EditorViewDelegate;

pub(crate) use line_draw::LineDraw;

/// Collected parameters for the draw pass.
pub struct DrawParams {
    pub(crate) h: u16,
    pub(crate) gutter_w: u16,
    pub(crate) avail: usize,
    pub(crate) scroll: usize,
    pub(crate) h_off: usize,
    pub(crate) tab_width: usize,
    pub(crate) wrap: bool,
    pub(crate) matchparen_pos: Option<(usize, usize)>,
    pub(crate) rainbow_maps: Vec<Vec<(usize, Color)>>,
}

/// Compute draw parameters from editor state and delegate.
pub fn build_draw_params<D: EditorViewDelegate>(editor: &Editor, delegate: &D, w: u16, h: u16) -> DrawParams {
    let gutter_w = compute_gutter_width(editor, delegate);
    let wrap = editor.options().wrap();
    let scroll = editor.viewport_scroll();
    let matchparen_pos = if editor.options().matchparen() {
        match_bracket(&editor.buf(), editor.cursor_line(), editor.cursor_col())
    } else {
        None
    };
    let total_lines = editor.buf().line_count();
    let viewport_end = (scroll + h as usize).min(total_lines);
    let rainbow_maps = rainbow::compute_rainbow_maps(editor, scroll, viewport_end);
    DrawParams {
        h,
        gutter_w,
        avail: w.saturating_sub(gutter_w) as usize,
        scroll,
        h_off: if wrap {
            0
        } else {
            editor.h_scroll()
        },
        tab_width: editor.options().tab_width(),
        wrap,
        matchparen_pos,
        rainbow_maps,
    }
}

/// Compute the total gutter width for an editor + delegate.
pub fn compute_gutter_width<D: EditorViewDelegate>(editor: &Editor, delegate: &D) -> u16 {
    let extra = delegate.extra_gutter_width();
    if !editor.options().number() {
        return extra;
    }
    let lines = editor.buf().line_count();
    let digits = digits_for(lines) as u16;
    digits + 1 + extra
}

/// Full editor draw pass into a buffer. This is the main reusable entry point.
pub fn draw_editor<D: EditorViewDelegate>(
    buf: &mut Buffer,
    editor: &Editor,
    delegate: &D,
    hl_cache: &mut HighlightCache,
    highlighter: &Highlighter,
) {
    let w = buf.width();
    let h = buf.height();
    if w == 0 || h == 0 {
        return;
    }

    buf.fill(' ', Style::default());
    let params = build_draw_params(editor, delegate, w, h);
    let total_lines = editor.buf().line_count();
    let viewport_end = (params.scroll + h as usize).min(total_lines);
    let spans = highlight_viewport(editor, hl_cache, highlighter, params.scroll, viewport_end);

    let sticky_lines = sticky::compute_sticky_lines(editor, params.scroll);
    for (i, sl) in sticky_lines.iter().enumerate() {
        sticky::draw_sticky_line(buf, sl, i as u16, params.gutter_w, w);
    }

    let row = draw_viewport_lines(buf, editor, delegate, &params, &spans, sticky_lines.len(), viewport_end);
    draw_tilde_fill(buf, row, h);
    prompt::draw_prompt(buf, editor, w, h);
}

fn draw_viewport_lines<D: EditorViewDelegate>(
    buf: &mut Buffer,
    editor: &Editor,
    delegate: &D,
    params: &DrawParams,
    spans: &[Vec<HlSpan>],
    start_row: usize,
    viewport_end: usize,
) -> usize {
    let mut row = start_row;
    let mut line_idx = params.scroll;
    while row < params.h as usize && line_idx < viewport_end {
        gutter::draw_gutter(buf, editor, delegate, line_idx, row as u16, params);
        let line_spans = spans.get(line_idx - params.scroll).map(|s| s.as_slice()).unwrap_or(&[]);
        let end_row = chars::draw_line_chars(buf, editor, delegate, line_idx, row, params, line_spans);
        row = end_row + 1;
        line_idx += 1;
    }
    row
}

fn draw_tilde_fill(buf: &mut Buffer, mut row: usize, h: u16) {
    let style = palette().style(StyleId::EditorGutter);
    let w = buf.width();
    while row < h as usize {
        buf.print_line(0, row as u16, "~", w, style);
        row += 1;
    }
}

fn highlight_viewport(
    editor: &Editor,
    hl_cache: &mut HighlightCache,
    highlighter: &Highlighter,
    scroll: usize,
    end: usize,
) -> Vec<Vec<HlSpan>> {
    let line_count = editor.buf().line_count();
    let editor_buf = editor.buf();
    hl_cache.highlight_viewport(
        scroll,
        end,
        line_count,
        |i| editor_buf.line(i).unwrap_or_default(),
        highlighter.syntax_set(),
        highlighter.theme(),
    )
}

// --- EditorView impl still uses these for its own View trait ---

impl<D: EditorViewDelegate> EditorView<D> {
    pub(super) fn gutter_width(&self) -> u16 {
        compute_gutter_width(&self.editor, &self.delegate)
    }

    pub(super) fn draw_impl(&mut self) {
        draw_editor(
            self.group.buffer_mut(),
            &self.editor,
            &self.delegate,
            &mut self.hl_cache,
            &self.highlighter,
        );
    }
}

fn digits_for(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    ((n as f64).log10().floor() as usize) + 1
}
