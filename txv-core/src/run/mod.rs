//! Backend trait, run loop, and MockBackend for tests.

pub mod mock;
mod wake_fd;
mod waker;

use std::time::Duration;

use crate::buffer::Buffer;
use crate::commands::CM_QUIT;
use crate::event::Event;
use crate::view::{EventSink, View};

pub use mock::{run_cycles, MockBackend};
pub use waker::Waker;

/// Backend trait — implemented by terminal renderers.
pub trait Backend: Send {
    fn poll_event(&mut self, timeout: Duration) -> Option<Event>;
    fn size(&self) -> (u16, u16);
    fn flush(&mut self, buffer: &Buffer);
    fn enter(&mut self);
    fn leave(&mut self);
    /// Force next flush to redraw all cells (bypass diff).
    fn invalidate(&mut self) {}
    /// Get a waker handle that background threads can use to interrupt poll_event.
    fn waker(&self) -> Waker {
        Waker::noop()
    }
    /// Show or hide the hardware cursor. Position is absolute (screen coords).
    fn set_cursor(&mut self, _cursor: Option<crate::cursor::CursorRequest>) {}
}

/// Run the main event loop. Returns when CM_QUIT is received.
pub fn run(root: &mut dyn View, backend: &mut dyn Backend) {
    backend.enter();
    let sink = EventSink::new();
    root.set_sink(sink.clone());

    loop {
        if root.needs_redraw() {
            root.render();
            backend.flush(root.buffer());
            backend.set_cursor(root.cursor());
        }

        if let Some(event) = backend.poll_event(Duration::from_millis(50)) {
            dispatch_event(root, &event, backend);
        } else {
            root.handle(&Event::Tick);
        }

        if drain_quit(root, &sink) {
            break;
        }
    }

    backend.leave();
}

fn dispatch_event(root: &mut dyn View, event: &Event, backend: &mut dyn Backend) {
    if let Event::Resize(nw, nh) = event {
        root.set_bounds(crate::geometry::Rect::new(0, 0, *nw, *nh));
        backend.invalidate();
    }
    root.handle(event);
}

fn drain_quit(root: &mut dyn View, sink: &EventSink) -> bool {
    let events = sink.drain();
    for ev in events {
        if let Event::Command { id, .. } = &ev {
            if *id == CM_QUIT {
                return true;
            }
        }
        root.handle(&ev);
    }
    false
}

#[cfg(test)]
mod tests;
