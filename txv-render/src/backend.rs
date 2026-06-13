//! CrosstermBackend — implements txv_core::Backend for crossterm terminals.

use std::io::{self, Write};
use std::time::Duration;

use crossterm::{cursor, event as ct_event, execute, queue, terminal};
use txv_core::buffer::Buffer;
use txv_core::cursor::{CursorRequest, CursorShape};
use txv_core::event::Event;
use txv_core::run::{Backend, Waker};

use crate::color::ColorMode;
use crate::event_translate::{translate_key, translate_mouse};
use crate::image_protocol::{detect_image_protocol, CellPixelSize, ImageProtocol};

/// Crossterm-based terminal backend with dual-buffer diffing.
pub struct CrosstermBackend {
    pub(crate) previous: Buffer,
    pub(crate) color_mode: ColorMode,
    pub(crate) force_full: bool,
    pub(crate) last_cursor: Option<CursorRequest>,
    pub(crate) cursor_dirty: bool,
    pub(crate) image_protocol: ImageProtocol,
    pub(crate) cell_size: CellPixelSize,
    wake_read: std::os::unix::io::RawFd,
    wake_write: std::os::unix::io::RawFd,
}

impl CrosstermBackend {
    pub fn new(color_mode: ColorMode) -> Self {
        let (w, h) = terminal::size().unwrap_or((80, 24));
        let mut fds = [0i32; 2];
        unsafe {
            libc::pipe(fds.as_mut_ptr());
        }
        // Make read end non-blocking
        unsafe {
            let flags = libc::fcntl(fds[0], libc::F_GETFL);
            libc::fcntl(fds[0], libc::F_SETFL, flags | libc::O_NONBLOCK);
        }
        Self {
            previous: Buffer::new(w, h),
            color_mode,
            force_full: true,
            last_cursor: None,
            cursor_dirty: true,
            image_protocol: detect_image_protocol(),
            cell_size: CellPixelSize::default(),
            wake_read: fds[0],
            wake_write: fds[1],
        }
    }

    /// Force next flush to emit all cells (no diff).
    pub fn invalidate(&mut self) {
        self.force_full = true;
    }
}

impl Backend for CrosstermBackend {
    fn enter(&mut self) {
        terminal::enable_raw_mode().ok();
        execute!(
            io::stdout(),
            terminal::EnterAlternateScreen,
            cursor::Hide,
            ct_event::EnableBracketedPaste
        )
        .ok();
    }

    fn leave(&mut self) {
        execute!(
            io::stdout(),
            ct_event::DisableBracketedPaste,
            cursor::Show,
            terminal::LeaveAlternateScreen
        )
        .ok();
        terminal::disable_raw_mode().ok();
    }

    fn size(&self) -> (u16, u16) {
        terminal::size().unwrap_or((80, 24))
    }

    fn poll_event(&mut self, timeout: Duration) -> Option<Event> {
        // Use libc::poll on both stdin and wake pipe
        let timeout_ms = timeout.as_millis() as i32;
        let mut fds = [
            libc::pollfd {
                fd: 0,
                events: libc::POLLIN,
                revents: 0,
            }, // stdin
            libc::pollfd {
                fd: self.wake_read,
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        let ready = unsafe { libc::poll(fds.as_mut_ptr(), 2, timeout_ms) };
        // Drain wake pipe if signaled
        if ready > 0 && fds[1].revents & libc::POLLIN != 0 {
            let mut buf = [0u8; 64];
            unsafe {
                libc::read(self.wake_read, buf.as_mut_ptr() as *mut libc::c_void, 64);
            }
        }
        // Always check crossterm — SIGWINCH queues Resize without stdin data
        if ct_event::poll(Duration::ZERO).unwrap_or(false) {
            match ct_event::read() {
                Ok(ct_event::Event::Key(k)) => return translate_key(k),
                Ok(ct_event::Event::Resize(w, h)) => return Some(Event::Resize(w, h)),
                Ok(ct_event::Event::Mouse(m)) => return translate_mouse(m),
                Ok(ct_event::Event::Paste(s)) => return Some(Event::Paste(s)),
                _ => {}
            }
        }
        if ready <= 0 {
            return None;
        }
        None
    }

    fn flush(&mut self, buf: &Buffer) {
        self.flush_buffer(buf);
        self.cursor_dirty = true;
    }

    fn invalidate(&mut self) {
        self.force_full = true;
    }

    fn waker(&self) -> Waker {
        Waker::from_fd(self.wake_write)
    }

    fn set_cursor(&mut self, req: Option<CursorRequest>) {
        let changed = self.last_cursor != req;
        if !changed && !self.cursor_dirty {
            return;
        }
        self.last_cursor = req;
        self.cursor_dirty = false;
        let mut out = io::stdout().lock();
        match req {
            Some(c) if c.shape() != CursorShape::Hidden => {
                queue!(out, cursor::MoveTo(c.x(), c.y()), cursor::Show).ok();
                if changed {
                    let seq = match c.shape() {
                        CursorShape::Block => "\x1b[2 q",
                        CursorShape::Underline => "\x1b[4 q",
                        CursorShape::Bar => "\x1b[6 q",
                        CursorShape::Hidden => unreachable!(),
                    };
                    out.write_all(seq.as_bytes()).ok();
                }
            }
            _ => {
                if changed {
                    queue!(out, cursor::Hide).ok();
                }
            }
        }
        out.flush().ok();
    }
}

impl Drop for CrosstermBackend {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.wake_read);
            libc::close(self.wake_write);
        }
    }
}

#[cfg(test)]
#[path = "backend_tests.rs"]
mod tests;
