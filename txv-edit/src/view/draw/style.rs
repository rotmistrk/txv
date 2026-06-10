//! Style composition — layered style for each character.

use txv_core::prelude::*;

use super::DrawParams;
use crate::editor::keymap::EditorMode;
use crate::editor::Editor;
use crate::view::delegate::{CursorRender, DecorationStyle, EditorViewDelegate};

/// Compose the final style for a character, applying all highlight layers.
pub fn compose_char_style<D: EditorViewDelegate>(
    editor: &Editor,
    delegate: &D,
    base: Style,
    line_idx: usize,
    char_idx: usize,
    byte_pos: usize,
    p: &DrawParams,
) -> Style {
    let style = apply_highlights(editor, delegate, base, line_idx, char_idx, byte_pos, p);
    let style = apply_delegate(delegate, style, line_idx, char_idx);
    let style = apply_delegate_ranges(delegate, style, line_idx, char_idx);
    let style = apply_delegate_decorations(delegate, style, line_idx, char_idx);
    apply_cursor(editor, delegate, style, line_idx, char_idx)
}

fn apply_cursor<D: EditorViewDelegate>(editor: &Editor, delegate: &D, style: Style, line: usize, col: usize) -> Style {
    if line != editor.cursor_line() || col != editor.cursor_col() {
        return style;
    }
    let mode = editor.mode();
    if !matches!(
        mode,
        EditorMode::Normal | EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock
    ) {
        return style;
    }
    match delegate.cursor_render(mode) {
        CursorRender::Software(cs) => cs,
        CursorRender::Hardware | CursorRender::None => {
            // Default software cursor: invert fg/bg
            let fg = if style.fg() == Color::Reset {
                Color::Rgb(220, 220, 220)
            } else {
                style.fg()
            };
            let bg = if style.bg() == Color::Reset {
                Color::Rgb(30, 30, 30)
            } else {
                style.bg()
            };
            Style::new(bg, fg).with_attrs(style.attrs())
        }
    }
}

fn apply_delegate_ranges<D: EditorViewDelegate>(delegate: &D, style: Style, line: usize, col: usize) -> Style {
    for hr in delegate.highlight_ranges(line) {
        if col >= hr.col_start && col < hr.col_end {
            let fg = if hr.style.fg() != Color::Reset {
                hr.style.fg()
            } else {
                style.fg()
            };
            let bg = if hr.style.bg() != Color::Reset {
                hr.style.bg()
            } else {
                style.bg()
            };
            return Style::new(fg, bg).with_attrs(style.attrs());
        }
    }
    style
}

fn apply_delegate_decorations<D: EditorViewDelegate>(delegate: &D, style: Style, line: usize, col: usize) -> Style {
    for dec in delegate.line_decorations(line) {
        if col >= dec.col_start && col < dec.col_end {
            return match &dec.style {
                DecorationStyle::Underline(_) | DecorationStyle::Squiggly(_) => {
                    style.with_attrs(style.attrs().underline())
                }
                DecorationStyle::Background(c) => Style::new(style.fg(), *c).with_attrs(style.attrs()),
            };
        }
    }
    style
}

fn apply_highlights<D: EditorViewDelegate>(
    editor: &Editor,
    delegate: &D,
    base: Style,
    line_idx: usize,
    char_idx: usize,
    byte_pos: usize,
    p: &DrawParams,
) -> Style {
    if in_visual_selection(editor, line_idx, char_idx) {
        let bg = palette().style(StyleId::VisualSelection).bg();
        return Style::new(base.fg(), bg).with_attrs(base.attrs());
    }
    if let Some(is_current) = editor.highlight().and_then(|h| h.match_at(byte_pos)) {
        return if is_current {
            delegate.highlight_match_style()
        } else {
            Style::new(base.fg(), delegate.highlight_other_bg()).with_attrs(base.attrs())
        };
    }
    apply_editor_decorations(editor, delegate, base, line_idx, char_idx, p)
}

fn apply_editor_decorations<D: EditorViewDelegate>(
    editor: &Editor,
    delegate: &D,
    mut style: Style,
    line_idx: usize,
    char_idx: usize,
    p: &DrawParams,
) -> Style {
    if style.bg() == Color::Reset && editor.ephemeral().ranges().iter().any(|r| r.covers_line(line_idx)) {
        style = Style::new(style.fg(), delegate.highlight_other_bg()).with_attrs(style.attrs());
    }
    if p.matchparen_pos == Some((line_idx, char_idx)) {
        let mp = delegate.matchparen_style();
        let fg = if mp.fg() != Color::Reset {
            mp.fg()
        } else {
            style.fg()
        };
        let bg = if mp.bg() != Color::Reset {
            mp.bg()
        } else {
            style.bg()
        };
        style = Style::new(fg, bg).with_attrs(mp.attrs());
    }
    if editor.options().rainbow() {
        if let Some(map) = p.rainbow_maps.get(line_idx.saturating_sub(p.scroll)) {
            if let Some(&(_, color)) = map.iter().find(|(col, _)| *col == char_idx) {
                style = style.with_fg(color);
            }
        }
    }
    style
}

fn apply_delegate<D: EditorViewDelegate>(delegate: &D, style: Style, line: usize, col: usize) -> Style {
    if let Some(extra) = delegate.extra_style(line, col) {
        extra
    } else {
        style
    }
}

fn in_visual_selection(editor: &Editor, line: usize, col: usize) -> bool {
    let mode = editor.mode();
    let Some((al, ac)) = editor.visual_anchor() else {
        return false;
    };
    let (cl, cc) = (editor.cursor_line(), editor.cursor_col());
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

fn ordered(al: usize, ac: usize, cl: usize, cc: usize) -> (usize, usize, usize, usize) {
    if al < cl || (al == cl && ac <= cc) {
        (al, ac, cl, cc)
    } else {
        (cl, cc, al, ac)
    }
}
