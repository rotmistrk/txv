//! draw() implementation for EditorView.

use txv_core::prelude::*;

use super::EditorView;
use crate::editor::keymap::EditorMode;
use crate::highlight::HlSpan;
use crate::view::delegate::EditorViewDelegate;

impl<D: EditorViewDelegate> EditorView<D> {
    pub(super) fn gutter_width(&self) -> u16 {
        if !self.editor.options().number() {
            return self.delegate.extra_gutter_width();
        }
        let lines = self.editor.buf().line_count();
        let digits = digits_for(lines) as u16;
        digits + 1 + self.delegate.extra_gutter_width()
    }

    pub(super) fn draw_impl(&mut self) {
        let h = self.state.buffer().height();
        let w = self.state.buffer().width();
        let gw = self.gutter_width();
        let text_w = w.saturating_sub(gw);

        self.state.buffer_mut().fill(' ', Style::default());
        if text_w == 0 || h == 0 {
            return;
        }

        let scroll = self.editor.viewport_scroll();
        let line_count = self.editor.buf().line_count();
        let spans = self.highlight_viewport(scroll, h as usize);

        for row in 0..h {
            let line_idx = scroll + row as usize;
            if line_idx >= line_count {
                break;
            }
            self.draw_gutter_line(line_idx, row, gw);
            self.draw_text_line(line_idx, row, gw, text_w, &spans);
        }
    }

    fn draw_gutter_line(&mut self, line: usize, row: u16, gw: u16) {
        let extra_gw = self.delegate.extra_gutter_width();
        let num_w = gw - extra_gw;
        if self.editor.options().number() && num_w > 0 {
            let s = format!("{:>w$} ", line + 1, w = (num_w - 1) as usize);
            let style = Style::new(Color::Ansi(8), Color::Reset);
            self.state.buffer_mut().print(0, row, &s, style);
        }
        let sign_x = num_w;
        let buf = self.state.buffer_mut();
        self.delegate.draw_gutter_sign(buf, line, sign_x, row);
    }

    fn draw_text_line(&mut self, line_idx: usize, row: u16, gw: u16, text_w: u16, spans: &[Vec<HlSpan>]) {
        let scroll = self.editor.viewport_scroll();
        let line_spans = &spans[line_idx - scroll];
        let h_scroll = self.editor.h_scroll();

        let mut col: usize = 0;
        let mut x: u16 = 0;
        for span in line_spans {
            let style = span.style();
            for ch in span.text().chars() {
                if ch == '\n' || ch == '\r' {
                    continue;
                }
                let cw = display_char_width(ch);
                if col >= h_scroll && x < text_w {
                    let final_style = self.compose_style(style, line_idx, col);
                    self.state.buffer_mut().put(gw + x, row, ch, final_style);
                    x += cw;
                }
                col += cw as usize;
            }
        }
    }

    fn compose_style(&self, base: Style, line: usize, col: usize) -> Style {
        if self.in_visual_selection(line, col) {
            let mut s = base;
            s.swap_fg_bg();
            return s;
        }
        if let Some(extra) = self.delegate.extra_style(line, col) {
            return extra;
        }
        base
    }

    fn in_visual_selection(&self, line: usize, col: usize) -> bool {
        let mode = self.editor.mode();
        let Some((al, ac)) = self.editor.visual_anchor() else {
            return false;
        };
        let (cl, cc) = (self.editor.cursor_line(), self.editor.cursor_col());
        match mode {
            EditorMode::Visual => {
                let (sl, sc, el, ec) = ordered(al, ac, cl, cc);
                if line < sl || line > el {
                    return false;
                }
                if sl == el {
                    return col >= sc && col <= ec;
                }
                if line == sl {
                    return col >= sc;
                }
                if line == el {
                    return col <= ec;
                }
                true
            }
            EditorMode::VisualLine => {
                let (sl, el) = (al.min(cl), al.max(cl));
                line >= sl && line <= el
            }
            EditorMode::VisualBlock => {
                let (sl, el) = (al.min(cl), al.max(cl));
                let (sc, ec) = (ac.min(cc), ac.max(cc));
                line >= sl && line <= el && col >= sc && col <= ec
            }
            _ => false,
        }
    }

    fn highlight_viewport(&mut self, scroll: usize, height: usize) -> Vec<Vec<HlSpan>> {
        let line_count = self.editor.buf().line_count();
        let end = (scroll + height).min(line_count);
        let editor_buf = self.editor.buf();
        self.hl_cache.highlight_viewport(
            scroll,
            end,
            line_count,
            |i| editor_buf.line(i).unwrap_or_default(),
            self.highlighter.syntax_set(),
            self.highlighter.theme(),
        )
    }
}

fn ordered(al: usize, ac: usize, cl: usize, cc: usize) -> (usize, usize, usize, usize) {
    if al < cl || (al == cl && ac <= cc) {
        (al, ac, cl, cc)
    } else {
        (cl, cc, al, ac)
    }
}

fn digits_for(n: usize) -> usize {
    if n == 0 {
        return 1;
    }
    ((n as f64).log10().floor() as usize) + 1
}
