//! InputDialog — modal dialog with a single-line text input.
//! Emits CM_OK with the entered text, or CM_CANCEL on Esc.
//!
//! Uses GroupState with an InputLine child for editing, so cursor
//! propagation and editing features (history, etc.) come for free.

use txv_core::prelude::*;

use crate::input_line::InputLine;

/// A modal prompt dialog with a title and single-line input.
pub struct InputDialog {
    group: GroupState,
    title_text: String,
}

impl InputDialog {
    pub fn new(title: impl Into<String>) -> Self {
        let title_text: String = title.into();
        let mut group = GroupState::new(ViewOptions {
            modal: true,
            focusable: true,
            ..ViewOptions::default()
        });
        let mut input = InputLine::new();
        input.select();
        group.insert(Box::new(input));
        group.set_focused_index(0);
        let mut s = Self { group, title_text };
        s.group.set_title(s.title_text.clone());
        s
    }

    /// Get the entered text.
    pub fn text(&mut self) -> &str {
        if let Some(il) = self.input_mut() {
            return il.text();
        }
        ""
    }

    fn input_mut(&mut self) -> Option<&mut InputLine> {
        self.group.child_mut(0)?.as_any_mut()?.downcast_mut::<InputLine>()
    }

    fn layout(&mut self) {
        let b = self.group.bounds();
        if b.w < 4 || b.h < 4 {
            return;
        }
        self.group
            .set_child_bounds(0, Rect::new(b.x + 2, b.y + 2, b.w.saturating_sub(4), 1));
    }
}

impl View for InputDialog {
    delegate_group_state!(group, override { set_bounds, draw, handle });

    fn set_bounds(&mut self, r: Rect) {
        self.group.set_bounds(r);
        self.layout();
    }

    fn draw(&mut self) {
        let w = self.group.buffer_mut().width();
        let h = self.group.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let normal = Style::default();
        let border = txv_core::palette::palette().style(StyleId::Border);
        let g = glyphs();
        let bx = &g.box_drawing;

        // Background
        for row in 0..h {
            self.group.buffer_mut().hline(0, row, w, ' ', normal);
        }
        // Border
        self.group.buffer_mut().hline(0, 0, w, bx.h_heavy, border);
        self.group
            .buffer_mut()
            .hline(0, h.saturating_sub(1), w, bx.h_heavy, border);
        for row in 1..h.saturating_sub(1) {
            self.group.buffer_mut().put(0, row, bx.v_heavy, border);
            self.group
                .buffer_mut()
                .put(w.saturating_sub(1), row, bx.v_heavy, border);
        }
        self.group.buffer_mut().put(0, 0, bx.tl_heavy, border);
        self.group.buffer_mut().put(w.saturating_sub(1), 0, bx.tr_heavy, border);
        self.group.buffer_mut().put(0, h.saturating_sub(1), bx.bl_heavy, border);
        self.group
            .buffer_mut()
            .put(w.saturating_sub(1), h.saturating_sub(1), bx.br_heavy, border);
        // Title
        if !self.title_text.is_empty() {
            let title = format!(" {} ", self.title_text);
            self.group.buffer_mut().print(2, 0, &title, border);
        }
        // Draw and blit InputLine child
        if let Some(child) = self.group.child_mut(0) {
            child.draw();
        }
        let (ox, oy) = self.group.child_origin(0);
        let buf_ptr = self.group.buffer_mut() as *mut Buffer;
        let child_buf = self.group.child(0).map(|c| c.buffer() as *const Buffer);
        if let Some(cbuf) = child_buf {
            unsafe { (*buf_ptr).blit(&*cbuf, ox, oy) };
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        self.group.dispatch(event)
    }
}
