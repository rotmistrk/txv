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
        let y = self.compute_cursor_y(line, scroll, gw, sticky_h)?;
        let x = gw + (col.saturating_sub(self.editor.h_scroll())) as u16;
        let shape = self.cursor_shape(mode)?;
        Some(CursorRequest::new(x, y, shape))
    }

    fn compute_cursor_y(&self, line: usize, scroll: usize, gw: u16, sticky_h: u16) -> Option<u16> {
        if self.editor.options().wrap() {
            let avail = self.group.bounds().w().saturating_sub(gw) as usize;
            if avail == 0 {
                return None;
            }
            let tw = self.editor.options().tab_width();
            let mut vrow = 0u16;
            for i in scroll..line {
                let l = self.editor.buf().line(i).unwrap_or_default();
                let w = display_width(&l, tw) as usize;
                vrow += if w == 0 {
                    1
                } else {
                    w.div_ceil(avail) as u16
                };
            }
            Some(vrow + sticky_h)
        } else {
            Some((line - scroll) as u16 + sticky_h)
        }
    }

    fn cursor_shape(&self, mode: EditorMode) -> Option<CursorShape> {
        let opts = self.editor.options();
        let cs = match mode {
            EditorMode::Insert => opts.cursor_insert(),
            _ => opts.cursor_normal(),
        };
        match cs {
            CursorStyle::Bar => Some(CursorShape::Bar),
            CursorStyle::Block => Some(CursorShape::Block),
            CursorStyle::Underline => Some(CursorShape::Underline),
            CursorStyle::Software => None,
        }
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
