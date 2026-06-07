//! Style composition — layered style for each character.

use txv_core::prelude::*;

use super::DrawParams;
use crate::editor::keymap::EditorMode;
use crate::editor::Editor;
use crate::view::delegate::EditorViewDelegate;

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
    let style = apply_highlights(editor, base, line_idx, char_idx, byte_pos, p);
    apply_delegate(delegate, style, line_idx, char_idx)
}

fn apply_highlights(
    editor: &Editor,
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
            palette().style(StyleId::SearchMatch)
        } else {
            let bg = palette().style(StyleId::CursorUnfocused).bg();
            Style::new(base.fg(), bg).with_attrs(base.attrs())
        };
    }
    apply_decorations(editor, base, line_idx, char_idx, p)
}

fn apply_decorations(editor: &Editor, mut style: Style, line_idx: usize, char_idx: usize, p: &DrawParams) -> Style {
    if style.bg() == Color::Reset && editor.ephemeral().ranges().iter().any(|r| r.covers_line(line_idx)) {
        let bg = palette().style(StyleId::SearchMatch).bg();
        style = Style::new(style.fg(), bg).with_attrs(style.attrs());
    }
    if p.matchparen_pos == Some((line_idx, char_idx)) {
        let mp = palette().style(StyleId::SearchMatch);
        style = Style::new(mp.fg(), mp.bg()).with_attrs(style.attrs().bold());
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
