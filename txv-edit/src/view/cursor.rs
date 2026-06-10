//! cursor() implementation for EditorView.

use txv_core::prelude::*;

use super::EditorView;
use crate::editor::keymap::EditorMode;
use crate::settings::CursorStyle;
use crate::view::delegate::{CursorRender, EditorViewDelegate};
use crate::view::draw::sticky::sticky_line_count;

impl<D: EditorViewDelegate> EditorView<D> {
    pub(super) fn cursor_impl(&self) -> Option<CursorRequest> {
        let mode = self.editor.mode();
        if self.cmdline_active {
            return self.cmdline_cursor();
        }
        // Check delegate cursor render preference
        match self.delegate.cursor_render(mode) {
            CursorRender::Software(_) | CursorRender::None => return None,
            CursorRender::Hardware => {}
        }
        let gw = self.gutter_width();
        let line = self.editor.cursor_line();
        let col = self.editor.cursor_col();
        let scroll = self.editor.viewport_scroll();
        if line < scroll {
            return None;
        }
        let sticky_h = sticky_line_count(&self.editor);
        let y = (line - scroll) as u16 + sticky_h;
        let x = gw + (col.saturating_sub(self.editor.h_scroll())) as u16;
        let opts = self.editor.options();
        let cs = match mode {
            EditorMode::Insert => opts.cursor_insert(),
            _ => opts.cursor_normal(),
        };
        if cs == CursorStyle::Software {
            return None;
        }
        let shape = match cs {
            CursorStyle::Bar => CursorShape::Bar,
            CursorStyle::Block => CursorShape::Block,
            CursorStyle::Underline => CursorShape::Underline,
            CursorStyle::Software => return None,
        };
        Some(CursorRequest::new(x, y, shape))
    }

    fn cmdline_cursor(&self) -> Option<CursorRequest> {
        if let Some(child) = self.group.focused_child() {
            if let Some(req) = child.cursor() {
                let (ox, oy) = self.group.child_origin(self.group.focused_index());
                let x = req.x().saturating_add(ox);
                let y = req.y().saturating_add(oy);
                return Some(CursorRequest::new(x, y, req.shape()));
            }
        }
        let h = self.group.bounds().h();
        Some(CursorRequest::new(1, h.saturating_sub(1), CursorShape::Bar))
    }
}
