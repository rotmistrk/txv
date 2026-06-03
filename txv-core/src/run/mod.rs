//! Backend trait, run loop, exec_view (modal), and MockBackend for tests.

pub mod mock;

use std::time::Duration;

use crate::buffer::Buffer;
use crate::commands::{CM_CANCEL, CM_CLOSE, CM_OK, CM_QUIT};
use crate::event::{CommandId, Event};
use crate::view::{EventSink, View};

pub use mock::{run_cycles, MockBackend};

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

/// A handle that wakes the event loop from any thread.
/// Clone + Send — safe to pass to background threads.
#[derive(Clone)]
pub struct Waker {
    fd: Option<std::sync::Arc<WakeFd>>,
}

struct WakeFd(std::os::unix::io::RawFd);

impl Drop for WakeFd {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.0);
        }
    }
}

// SAFETY: the fd is only written to (single byte), which is atomic on pipes.
unsafe impl Send for WakeFd {}
unsafe impl Sync for WakeFd {}

impl Waker {
    /// Create a waker from the write end of a pipe.
    pub fn from_fd(write_fd: std::os::unix::io::RawFd) -> Self {
        Self {
            fd: Some(std::sync::Arc::new(WakeFd(write_fd))),
        }
    }

    /// No-op waker (for tests/mock backends).
    pub fn noop() -> Self {
        Self { fd: None }
    }

    /// Wake the event loop. Safe to call from any thread.
    pub fn wake(&self) {
        if let Some(fd) = &self.fd {
            unsafe {
                libc::write(fd.0, b"W".as_ptr() as *const libc::c_void, 1);
            }
        }
    }
}

/// Run the main event loop. Returns when CM_QUIT is received.
pub fn run(root: &mut dyn View, backend: &mut dyn Backend) {
    backend.enter();
    let sink = EventSink::new();
    root.set_sink(sink.clone());

    loop {
        if root.needs_redraw() {
            root.draw();
            root.mark_redrawn();
            backend.flush(root.buffer());
            backend.set_cursor(root.cursor());
        }

        if let Some(event) = backend.poll_event(Duration::from_millis(50)) {
            if let Event::Resize(nw, nh) = &event {
                root.set_bounds(crate::geometry::Rect::new(0, 0, *nw, *nh));
                backend.invalidate();
            }
            root.handle(&event);
        } else {
            root.handle(&Event::Tick);
        }

        let events = sink.drain();
        for ev in events {
            if let Event::Command { id, .. } = &ev {
                if *id == CM_QUIT {
                    backend.leave();
                    return;
                }
            }
            root.handle(&ev);
        }
    }
}

/// Modal nested event loop. Returns the closing command (CM_CLOSE, CM_OK, or CM_CANCEL).
pub fn exec_view(root: &mut dyn View, modal: &mut dyn View, backend: &mut dyn Backend) -> CommandId {
    let sink = EventSink::new();
    modal.set_sink(sink.clone());

    loop {
        // Draw root, then composite modal on top
        root.draw();
        modal.draw();
        let rb = root.bounds();
        let mut combined = Buffer::new(rb.w, rb.h);
        combined.blit(root.buffer(), 0, 0);
        // Modal is composited at (0,0) — caller should size it to desired position
        combined.blit(modal.buffer(), 0, 0);
        backend.flush(&combined);
        // Hardware cursor from modal
        let cursor = modal.cursor();
        backend.set_cursor(cursor);

        match backend.poll_event(Duration::from_millis(50)) {
            Some(Event::Key(ref k)) => {
                modal.handle(&Event::Key(*k));
            }
            Some(Event::Mouse(m)) => {
                modal.handle(&Event::Mouse(m));
            }
            Some(Event::Resize(nw, nh)) => {
                let ev = Event::Resize(nw, nh);
                root.handle(&ev);
                modal.handle(&ev);
            }
            Some(Event::Tick) | None => {
                root.handle(&Event::Tick);
                modal.handle(&Event::Tick);
            }
            Some(ev @ Event::Command { .. }) => {
                root.handle(&ev);
            }
            Some(ev @ Event::Paste(_)) => {
                modal.handle(&ev);
            }
        }

        let events = sink.drain();
        for ev in events {
            if let Event::Command { id, .. } = &ev {
                if matches!(*id, CM_CLOSE | CM_OK | CM_CANCEL) {
                    return *id;
                }
            }
            root.handle(&ev);
        }
    }
}

#[cfg(test)]
mod tests;
