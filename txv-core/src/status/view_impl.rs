//! View trait implementation for StatusBar.

use crate::buffer::Buffer;
use crate::cell::Style;
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
        let bar_style = crate::palette::palette().chrome().status_bar();
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
            entries.push(LayoutEntry {
                idx,
                text_w: text.len() as u16,
                text,
                gravity: self.item_gravity(idx),
                priority: self.item_priority(idx),
                stretch: self.item_stretch(idx),
                max_w: self.item_max_width(idx),
                alloc: 0,
            });
        }

        // Phase 1: drop lowest-priority items until total min fits
        entries.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.idx.cmp(&b.idx)));
        let mut total: u16 = entries.iter().map(|e| e.text_w).sum();
        while total > w && !entries.is_empty() {
            total -= entries.last().map(|e| e.text_w).unwrap_or(0);
            entries.pop();
        }

        // Phase 2: allocate min-sz, distribute remaining to stretch items
        for e in &mut entries {
            e.alloc = e.text_w;
        }
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

        // Phase 3: restore insertion order and render
        entries.sort_by_key(|e| e.idx);

        let mut lx: u16 = 0;
        // Compute right-side total to know where right items start
        let right_total: u16 = entries
            .iter()
            .filter(|e| e.gravity == Gravity::Right)
            .map(|e| e.alloc)
            .sum();
        let mut rx = w.saturating_sub(right_total);

        for e in &entries {
            match e.gravity {
                Gravity::Left => {
                    if lx + e.alloc <= rx {
                        let style = self.render_style(e.idx, bar_style);
                        self.state.buffer_mut().print_line(lx, 0, &e.text, e.alloc, style);
                        lx += e.alloc;
                    }
                }
                Gravity::Right => {
                    let style = self.render_style(e.idx, bar_style);
                    self.state.buffer_mut().print_line(rx, 0, &e.text, e.alloc, style);
                    rx += e.alloc;
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
                bg: bar_style.bg,
                attrs: s.attrs,
            }
        } else {
            bar_style
        }
    }
}

struct LayoutEntry {
    idx: usize,
    text: String,
    text_w: u16,
    gravity: Gravity,
    priority: u8,
    stretch: u16,
    max_w: u16,
    alloc: u16,
}
