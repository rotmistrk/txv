//! View trait implementation for StatusBar.

use crate::buffer::Buffer;
use crate::cell::{Attrs, Color, Style};
use crate::event::Event;
use crate::geometry::Rect;
use crate::view::{EventSink, HandleResult, View, ViewOptions};

use super::{Gravity, ItemSlot, StatusBar};

impl View for StatusBar {
    fn bounds(&self) -> Rect {
        self.state.bounds()
    }
    fn set_bounds(&mut self, rect: Rect) {
        self.state.set_bounds(rect);
        self.state.mark_dirty();
    }
    fn set_sink(&mut self, sink: EventSink) {
        self.state.set_sink(sink);
    }
    fn options(&self) -> ViewOptions {
        self.state.options
    }
    fn title(&self) -> &str {
        ""
    }
    fn needs_redraw(&self) -> bool {
        self.state.is_dirty()
    }
    fn mark_redrawn(&mut self) {
        self.state.mark_redrawn();
    }
    fn select(&mut self) {}
    fn unselect(&mut self) {}

    fn draw(&mut self) {
        let w = self.state.buf.width();
        let h = self.state.buf.height();
        if w == 0 || h == 0 {
            return;
        }
        let style = Style {
            attrs: Attrs {
                reverse: true,
                ..Attrs::default()
            },
            ..Style::default()
        };
        self.state.buf.hline(0, 0, w, ' ', style);

        if let Some(idx) = self.exclusive {
            if let Some(label) = self.visible_label(idx) {
                let label = label.to_string();
                self.state.buf.print_line(0, 0, &label, w, style);
            }
            return;
        }

        // Collect items info first (avoids borrow conflict with buf)
        struct ItemInfo {
            text: String,
            gravity: Gravity,
            item_style: Style,
        }
        let mut items: Vec<ItemInfo> = Vec::new();
        for slot in &self.items {
            let label = match slot {
                ItemSlot::Full(item) => item.label(),
                ItemSlot::VisibleOnly(item) => item.label(),
                ItemSlot::ActiveOnly(_) => continue,
            };
            if label.is_empty() {
                continue;
            }
            let gravity = match slot {
                ItemSlot::Full(item) => item.gravity(),
                ItemSlot::VisibleOnly(item) => item.gravity(),
                ItemSlot::ActiveOnly(_) => continue,
            };
            let item_style = match slot {
                ItemSlot::Full(item) => item.style(),
                ItemSlot::VisibleOnly(item) => item.style(),
                ItemSlot::ActiveOnly(_) => Style::default(),
            };
            items.push(ItemInfo {
                text: format!(" {label} "),
                gravity,
                item_style,
            });
        }

        // Render left items
        let mut lx: u16 = 0;
        for item in &items {
            if item.gravity == Gravity::Left {
                let tw = item.text.len() as u16;
                if lx + tw <= w {
                    self.state.buf.print(lx, 0, &item.text, style);
                    lx += tw;
                }
            }
        }

        // Render right items from right edge
        let mut rx = w;
        for item in items.iter().rev() {
            if item.gravity == Gravity::Right {
                let tw = item.text.len() as u16;
                if rx >= tw && rx - tw >= lx {
                    rx -= tw;
                    let s = if item.item_style.fg != Color::default() {
                        Style {
                            fg: item.item_style.fg,
                            attrs: style.attrs,
                            ..Style::default()
                        }
                    } else {
                        style
                    };
                    self.state.buf.print(rx, 0, &item.text, s);
                }
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Tick = event {
            self.tick_items();
            return HandleResult::Ignored;
        }

        let sink = match self.state.sink() {
            Some(s) => s.clone(),
            None => return HandleResult::Ignored,
        };

        if let Some(idx) = self.exclusive {
            let result = self.handle_active(idx, event, &sink);
            // Check if item released exclusive
            if !self.item_is_exclusive(idx) {
                self.exclusive = None;
                self.state.mark_dirty();
            }
            return result;
        }

        // Route to all active items, first consumed wins
        for i in 0..self.items.len() {
            let result = self.handle_active(i, event, &sink);
            if result == HandleResult::Consumed {
                // Check if item claimed exclusive
                if self.item_is_exclusive(i) {
                    self.exclusive = Some(i);
                    self.state.mark_dirty();
                }
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }

    fn buffer(&self) -> &Buffer {
        &self.state.buf
    }
}
