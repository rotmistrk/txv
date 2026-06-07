//! Command/search prompt rendering at the bottom of the editor.

use txv_core::prelude::*;

use crate::editor::keymap::EditorMode;
use crate::editor::Editor;

pub fn draw_prompt(buf: &mut Buffer, editor: &Editor, w: u16, h: u16) {
    let mode = editor.mode();
    if mode != EditorMode::Command && mode != EditorMode::Search {
        return;
    }
    let y = h.saturating_sub(1);
    let style = palette().style(StyleId::StatusBar);
    let prefix = if mode == EditorMode::Search {
        "/"
    } else {
        ":"
    };
    let text = format!("{}{}", prefix, editor.command_buf());
    buf.print_line(0, y, &text, w, style);
    let cx = text.len() as u16;
    if cx < w {
        let cs = palette().style(StyleId::InputCursor);
        buf.put(cx, y, ' ', cs);
    }
}
