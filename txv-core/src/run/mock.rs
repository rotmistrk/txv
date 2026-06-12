//! MockBackend for testing without a terminal.

use std::time::Duration;

use crate::buffer::Buffer;
use crate::commands::CM_QUIT;
use crate::event::{Event, KeyCode, KeyEvent, KeyMod};
use crate::geometry::Rect;
use crate::view::{EventSink, View};

use super::Backend;

/// Run N event-loop cycles with a MockBackend (for testing).
pub fn run_cycles(root: &mut dyn View, backend: &mut MockBackend, n: usize) {
    let sink = EventSink::new();
    root.set_sink(sink.clone());

    for _ in 0..n {
        if pump_events(root, backend, &sink) {
            return;
        }
        if drain_sink(root, &sink) {
            return;
        }
        root.render();
        backend.flush(root.buffer());
    }
}

fn pump_events(root: &mut dyn View, backend: &mut MockBackend, sink: &EventSink) -> bool {
    while let Some(event) = backend.poll_event(Duration::ZERO) {
        if let Event::Resize(nw, nh) = &event {
            root.set_bounds(Rect::new(0, 0, *nw, *nh));
        }
        root.handle(&event);
        if drain_sink(root, sink) {
            return true;
        }
    }
    false
}

/// Drain all pending sink events (may produce further events). Returns true on CM_QUIT.
fn drain_sink(root: &mut dyn View, sink: &EventSink) -> bool {
    loop {
        let events = sink.drain();
        if events.is_empty() {
            return false;
        }
        for ev in events {
            if let Event::Command { id: CM_QUIT, .. } = &ev {
                return true;
            }
            root.handle(&ev);
        }
    }
}

/// Mock backend for testing without a terminal.
pub struct MockBackend {
    width: u16,
    height: u16,
    events: Vec<Event>,
    last_buffer: Option<Buffer>,
    last_cursor: Option<crate::cursor::CursorRequest>,
}

impl MockBackend {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            events: Vec::new(),
            last_buffer: None,
            last_cursor: None,
        }
    }

    pub fn inject(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn inject_key(&mut self, code: KeyCode, modifiers: KeyMod) {
        self.inject(Event::Key(KeyEvent { code, modifiers }));
    }

    pub fn inject_str(&mut self, s: &str) {
        for ch in s.chars() {
            match ch {
                '\n' => self.inject_key(KeyCode::Enter, KeyMod::default()),
                '\x1b' => self.inject_key(KeyCode::Esc, KeyMod::default()),
                '\t' => self.inject_key(KeyCode::Tab, KeyMod::default()),
                c => self.inject_key(KeyCode::Char(c), KeyMod::default()),
            }
        }
    }

    pub fn inject_paste(&mut self, text: &str) {
        self.inject(Event::Paste(text.to_string()));
    }

    /// Resize the mock terminal and inject a Resize event.
    pub fn set_size(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        self.inject(Event::Resize(width, height));
    }

    pub fn buffer(&self) -> Option<&Buffer> {
        self.last_buffer.as_ref()
    }

    pub fn screen_text(&self) -> String {
        let Some(ref buf) = self.last_buffer else {
            return String::new();
        };
        let mut rows = Vec::new();
        for y in 0..buf.height() {
            rows.push(self.row(y));
        }
        rows.join("\n")
    }

    /// Check if text appears anywhere on screen (including status bar).
    pub fn contains(&self, text: &str) -> bool {
        let Some(ref buf) = self.last_buffer else {
            return false;
        };
        for y in 0..buf.height() {
            if self.row(y).contains(text) {
                return true;
            }
        }
        false
    }

    /// Check if text appears in the content area (excludes status bar on last row).
    pub fn content_contains(&self, text: &str) -> bool {
        let Some(ref buf) = self.last_buffer else {
            return false;
        };
        let content_rows = buf.height().saturating_sub(1);
        for y in 0..content_rows {
            if self.row(y).contains(text) {
                return true;
            }
        }
        false
    }

    pub fn row(&self, y: u16) -> String {
        let Some(ref buf) = self.last_buffer else {
            return String::new();
        };
        if y >= buf.height() {
            return String::new();
        }
        let mut row = String::new();
        for x in 0..buf.width() {
            row.push(buf.cell(x, y).ch());
        }
        row.trim_end().to_string()
    }

    /// Last cursor request set by the program.
    pub fn cursor(&self) -> Option<crate::cursor::CursorRequest> {
        self.last_cursor
    }
}

impl Backend for MockBackend {
    fn poll_event(&mut self, _timeout: Duration) -> Option<Event> {
        if self.events.is_empty() {
            None
        } else {
            Some(self.events.remove(0))
        }
    }
    fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }
    fn flush(&mut self, buffer: &Buffer) {
        let mut copy = Buffer::new(buffer.width(), buffer.height());
        for y in 0..buffer.height() {
            for x in 0..buffer.width() {
                let cell = buffer.cell(x, y);
                copy.put(x, y, cell.ch, cell.style);
            }
        }
        self.last_buffer = Some(copy);
    }
    fn set_cursor(&mut self, cursor: Option<crate::cursor::CursorRequest>) {
        self.last_cursor = cursor;
    }
    fn enter(&mut self) {}
    fn leave(&mut self) {}
}
