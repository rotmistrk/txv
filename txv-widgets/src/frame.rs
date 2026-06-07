//! Frame — a Group that wraps a single child in a box-drawing border.
//!
//! Supports text labels at six positions on the border:
//! lt (left-top), t (center-top), rt (right-top),
//! lb (left-bottom), b (center-bottom), rb (right-bottom).

use txv_core::palette::palette;
use txv_core::prelude::*;

/// Border label position.
#[derive(Clone, Copy)]
pub enum FrameLabel {
    LeftTop = 0,
    Top = 1,
    RightTop = 2,
    LeftBottom = 3,
    Bottom = 4,
    RightBottom = 5,
}

/// A Group that draws a box border around a single child.
pub struct Frame {
    group: GroupState,
    labels: [String; 6],
}

impl Frame {
    pub fn new(child: Box<dyn View>) -> Self {
        let mut group = GroupState::new(ViewOptions::default());
        group.insert(child);
        group.set_focused_index(0);
        Self {
            group,
            labels: Default::default(),
        }
    }

    pub fn set_label(&mut self, pos: FrameLabel, text: impl Into<String>) {
        self.labels[pos as usize] = text.into();
    }

    fn layout(&mut self) {
        let b = self.group.bounds();
        if b.w() > 2 && b.h() > 2 {
            self.group.set_child_bounds(0, Rect::new(1, 1, b.w() - 2, b.h() - 2));
        } else {
            self.group.set_child_bounds(0, Rect::new(0, 0, 0, 0));
        }
    }

    fn draw_border(&mut self) {
        let w = self.group.buffer_mut().width();
        let h = self.group.buffer_mut().height();
        if w < 2 || h < 2 {
            return;
        }
        let style = palette().style(StyleId::Border);
        let g = glyphs();
        let bx = &g.box_drawing();

        self.group.buffer_mut().hline(0, 0, w, bx.h(), style);
        self.group.buffer_mut().hline(0, h - 1, w, bx.h(), style);
        for row in 1..h - 1 {
            self.group.buffer_mut().put(0, row, bx.v(), style);
            self.group.buffer_mut().put(w - 1, row, bx.v(), style);
        }
        self.group.buffer_mut().put(0, 0, bx.tl(), style);
        self.group.buffer_mut().put(w - 1, 0, bx.tr(), style);
        self.group.buffer_mut().put(0, h - 1, bx.bl(), style);
        self.group.buffer_mut().put(w - 1, h - 1, bx.br(), style);

        self.draw_border_labels(w, h, style);
    }

    fn draw_border_labels(&mut self, w: u16, h: u16, style: Style) {
        for (i, text) in self.labels.iter().enumerate() {
            if text.is_empty() {
                continue;
            }
            let avail = (w as usize).saturating_sub(4);
            if avail == 0 {
                continue;
            }
            let display = if text.len() > avail {
                &text[..avail]
            } else {
                text.as_str()
            };
            let y = match i {
                0..=2 => 0,
                _ => h - 1,
            };
            let x = match i {
                0 | 3 => 2,
                2 | 5 => w.saturating_sub(display.len() as u16 + 2),
                _ => (w.saturating_sub(display.len() as u16)) / 2,
            };
            self.group.buffer_mut().print(x, y, display, style);
        }
    }
}

impl View for Frame {
    delegate_group_state!(group, override { set_bounds, draw });

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
        let bg = palette().style(StyleId::StatusBar);
        self.group.buffer_mut().fill(' ', bg);
        self.draw_border();

        // Draw and blit child
        if let Some(child) = self.group.child_mut(0) {
            if child.bounds().w() > 0 && child.bounds().h() > 0 {
                child.draw();
            }
        }
        let buf_ptr = self.group.buffer_mut() as *mut Buffer;
        if let Some(child) = self.group.child(0) {
            if child.bounds().w() > 0 {
                let (ox, oy) = self.group.child_origin(0);
                unsafe { (*buf_ptr).blit(child.buffer(), ox, oy) };
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        self.group.dispatch(event)
    }
}
