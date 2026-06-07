//! Sticky scroll — pinned scope headers at top of viewport.

use txv_core::prelude::*;

use crate::editor::Editor;

pub(crate) struct StickyLine {
    pub(crate) text: String,
}

/// Find scope headers above viewport. Returns at most 2 lines.
pub(crate) fn compute_sticky_lines(editor: &Editor, scroll: usize) -> Vec<StickyLine> {
    if scroll == 0 {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut current_indent = indent_of_line(editor, scroll);

    for line_idx in (0..scroll).rev() {
        let line = editor.buf().line(line_idx).unwrap_or_default();
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        let indent = line.len() - trimmed.len();
        if indent < current_indent && is_scope_header(trimmed) {
            result.push(StickyLine { text: line });
            current_indent = indent;
            if result.len() >= 2 {
                break;
            }
        }
    }
    result.reverse();
    result
}

pub(crate) fn draw_sticky_line(buf: &mut Buffer, sl: &StickyLine, y: u16, w: u16) {
    let style = palette().style(StyleId::Dim);
    buf.hline(0, y, w, ' ', style);
    let text: String = sl.text.chars().take(w as usize).collect();
    buf.print(0, y, &text, style);
}

fn indent_of_line(editor: &Editor, line_idx: usize) -> usize {
    let line = editor.buf().line(line_idx).unwrap_or_default();
    let trimmed = line.trim_start();
    if trimmed.is_empty() {
        4
    } else {
        line.len() - trimmed.len()
    }
}

fn is_scope_header(trimmed: &str) -> bool {
    trimmed.starts_with("fn ")
        || trimmed.starts_with("pub fn ")
        || trimmed.starts_with("pub(crate) fn ")
        || trimmed.starts_with("pub(super) fn ")
        || trimmed.starts_with("impl ")
        || trimmed.starts_with("pub struct ")
        || trimmed.starts_with("struct ")
        || trimmed.starts_with("pub enum ")
        || trimmed.starts_with("enum ")
        || trimmed.starts_with("mod ")
        || trimmed.starts_with("pub mod ")
        || trimmed.starts_with("trait ")
        || trimmed.starts_with("pub trait ")
        || trimmed.starts_with("def ")
        || trimmed.starts_with("class ")
        || trimmed.starts_with("async def ")
        || trimmed.starts_with("function ")
        || trimmed.starts_with("export function ")
        || trimmed.starts_with("export class ")
        || trimmed.starts_with("func ")
        || trimmed.starts_with("type ")
}
