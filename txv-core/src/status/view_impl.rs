//! View trait implementation for StatusBar.

use crate::buffer::Buffer;
use crate::cell::{Attrs, Style};
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
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let bar_style = Style {
            attrs: Attrs {
                reverse: true,
                ..Attrs::default()
            },
            ..Style::default()
        };
        self.state.buffer_mut().hline(0, 0, w, ' ', bar_style);

        if let Some(idx) = self.exclusive {
            if let Some(label) = self.visible_label(idx) {
                let label = label.to_string();
                self.state.buffer_mut().print_line(0, 0, &label, w, bar_style);
            }
            return;
        }

        // Collect visible items
        let mut entries: Vec<LayoutEntry> = Vec::new();
        for (idx, slot) in self.items.iter().enumerate() {
            let label = match slot {
                ItemSlot::Full(item) => item.label(),
                ItemSlot::VisibleOnly(item) => item.label(),
                ItemSlot::ActiveOnly(_) => continue,
            };
            if label.is_empty() {
                continue;
            }
            let text = format!(" {} ", label);
            let text_w = text.len() as u16;
            entries.push(LayoutEntry {
                idx,
                text,
                gravity: self.item_gravity(idx),
                priority: self.item_priority(idx),
                stretch: self.item_stretch(idx),
                max_w: self.item_max_width(idx),
                alloc: text_w,
            });
        }

        // Drop lowest-priority items if total exceeds width
        let total: u16 = entries.iter().map(|e| e.alloc).sum();
        if total > w {
            entries.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.idx.cmp(&b.idx)));
            let mut sum = 0u16;
            let mut keep = entries.len();
            for (i, e) in entries.iter().enumerate() {
                if sum + e.alloc > w {
                    keep = i;
                    break;
                }
                sum += e.alloc;
            }
            entries.truncate(keep);
            entries.sort_by_key(|e| e.idx);
        }

        // Distribute remaining space to stretch items
        let used: u16 = entries.iter().map(|e| e.alloc).sum();
        let remaining = w.saturating_sub(used);
        if remaining > 0 {
            let total_stretch: u16 = entries.iter().map(|e| e.stretch).sum();
            if total_stretch > 0 {
                for e in &mut entries {
                    if e.stretch > 0 {
                        let share = (remaining as u32 * e.stretch as u32 / total_stretch as u32) as u16;
                        let capped = if e.max_w > 0 {
                            share.min(e.max_w.saturating_sub(e.alloc))
                        } else {
                            share
                        };
                        e.alloc += capped;
                    }
                }
            }
        }

        // Render right-gravity items first (they have priority over left)
        let mut rx = w;
        for e in entries.iter().rev() {
            if e.gravity == Gravity::Right && rx >= e.alloc {
                rx -= e.alloc;
                let style = self.render_style(e.idx, bar_style);
                self.state.buffer_mut().print_line(rx, 0, &e.text, e.alloc, style);
            }
        }

        // Render left-gravity items (stop before right items)
        let mut lx: u16 = 0;
        for e in &entries {
            if e.gravity == Gravity::Left {
                let avail = rx.saturating_sub(lx);
                if avail == 0 {
                    break;
                }
                let use_w = e.alloc.min(avail);
                let style = self.render_style(e.idx, bar_style);
                self.state.buffer_mut().print_line(lx, 0, &e.text, use_w, style);
                lx += use_w;
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
            if !self.item_is_exclusive(idx) {
                self.exclusive = None;
                self.state.mark_dirty();
            }
            return result;
        }

        for i in 0..self.items.len() {
            let result = self.handle_active(i, event, &sink);
            if result == HandleResult::Consumed {
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
        self.state.buffer()
    }
}

impl StatusBar {
    fn render_style(&self, idx: usize, bar_style: Style) -> Style {
        let s = self.item_style(idx);
        if s.fg != crate::cell::Color::default() {
            Style {
                fg: s.fg,
                attrs: bar_style.attrs,
                ..Style::default()
            }
        } else {
            bar_style
        }
    }
}

struct LayoutEntry {
    idx: usize,
    text: String,
    gravity: Gravity,
    priority: u8,
    stretch: u16,
    max_w: u16,
    alloc: u16,
}
