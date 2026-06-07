//! Rainbow bracket computation.

use txv_core::prelude::Color;

use crate::editor::Editor;

pub const RAINBOW_COLORS: [Color; 4] = [
    Color::Ansi(3), // yellow
    Color::Ansi(5), // magenta
    Color::Ansi(6), // cyan
    Color::Ansi(2), // green
];

/// Compute rainbow bracket maps for the viewport.
pub(crate) fn compute_rainbow_maps(editor: &Editor, scroll: usize, viewport_end: usize) -> Vec<Vec<(usize, Color)>> {
    if !editor.options().rainbow() {
        return Vec::new();
    }
    let mut depth = bracket_depth_at_line(editor, scroll);
    let mut maps = Vec::with_capacity(viewport_end - scroll);
    for i in scroll..viewport_end {
        let line = editor.buf().line(i).unwrap_or_default();
        let (map, new_depth) = rainbow_brackets_with_depth(&line, depth);
        maps.push(map);
        depth = new_depth;
    }
    maps
}

fn bracket_depth_at_line(editor: &Editor, line: usize) -> usize {
    let mut depth: usize = 0;
    for i in 0..line {
        let text = editor.buf().line(i).unwrap_or_default();
        for ch in text.chars() {
            match ch {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
    }
    depth
}

pub fn rainbow_brackets_with_depth(line: &str, mut depth: usize) -> (Vec<(usize, Color)>, usize) {
    let mut result = Vec::new();
    for (idx, ch) in line.chars().enumerate() {
        match ch {
            '(' | '[' | '{' => {
                result.push((idx, RAINBOW_COLORS[depth % RAINBOW_COLORS.len()]));
                depth += 1;
            }
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
                result.push((idx, RAINBOW_COLORS[depth % RAINBOW_COLORS.len()]));
            }
            _ => {}
        }
    }
    (result, depth)
}
