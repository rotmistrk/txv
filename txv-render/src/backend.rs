//! CrosstermBackend — implements txv_core::Backend for crossterm terminals.

use std::io::{self, Write};
use std::time::Duration;

use crossterm::{
    cursor, event as ct_event, execute, queue,
    style::{self, Attribute, SetAttribute},
    terminal::{self, Clear, ClearType},
};
use txv_core::buffer::Buffer;
use txv_core::cell::Style;
use txv_core::cursor::{CursorRequest, CursorShape};
use txv_core::event::Event;
use txv_core::run::{Backend, Waker};

use crate::color::ColorMode;
use crate::event_translate::{translate_key, translate_mouse};
use crate::style_emit::{apply_color_mode, emit_style};

/// Crossterm-based terminal backend with dual-buffer diffing.
pub struct CrosstermBackend {
    previous: Buffer,
    color_mode: ColorMode,
    force_full: bool,
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

    fn flush(&mut self, surface: &Buffer) {
        let w = surface.width();
        let h = surface.height();

        // Resize or force-full: invalidate previous buffer so all cells are emitted
        if self.previous.width() != w || self.previous.height() != h {
            self.previous = Buffer::new(w, h);
            self.force_full = true;
        }

        let mut out = io::stdout().lock();
        let mut last_style: Option<Style> = None;

        if self.force_full {
            queue!(out, SetAttribute(Attribute::Reset)).ok();
            queue!(out, Clear(ClearType::All)).ok();
            last_style = None;
        }

        for y in 0..h {
            let mut cursor_x: Option<u16> = None;

            // Find last meaningful cell on this row (non-default-space).
            let last_meaningful = (0..w)
                .rev()
                .find(|&x| {
                    let c = surface.cell(x, y);
                    c.ch != ' ' || c.style != Style::default() || c.width != 1
                })
                .map(|x| x + 1)
                .unwrap_or(0);

            let mut x = 0u16;
            while x < w {
                let cell = surface.cell(x, y);
                let prev = self.previous.cell(x, y);

                // Beyond last meaningful: only emit EL if previous had content here
                if x >= last_meaningful {
                    // Check if any remaining previous cells are non-default-space
                    if !self.force_full {
                        let prev_had_content = (x..w).any(|px| {
                            let p = self.previous.cell(px, y);
                            p.ch != ' ' || p.style != Style::default() || p.width != 1
                        });
                        if prev_had_content {
                            queue!(out, cursor::MoveTo(x, y)).ok();
                            queue!(out, SetAttribute(Attribute::Reset)).ok();
                            queue!(out, Clear(ClearType::UntilNewLine)).ok();
                            last_style = None;
                        }
                    }
                    break;
                }

                // Skip unchanged cells — but NEVER skip if previous frame had
                // a wide char or placeholder here (terminal state may differ)
                if !self.force_full
                    && cell.ch == prev.ch
                    && cell.style == prev.style
                    && cell.width == prev.width
                    && prev.width == 1
                {
                    x += 1;
                    cursor_x = None;
                    continue;
                }

                // Skip continuation cells that haven't changed
                if cell.width == 0 && prev.width == 0 {
                    x += 1;
                    continue;
                }

                // Need to emit this cell
                if cursor_x != Some(x) {
                    queue!(out, cursor::MoveTo(x, y)).ok();
                }

                let style = apply_color_mode(cell.style, self.color_mode);
                if last_style.as_ref() != Some(&style) {
                    emit_style(&mut out, &style);
                    last_style = Some(style);
                }

                let ch = if cell.width == 0 {
                    ' '
                } else {
                    cell.ch
                };
                queue!(out, style::Print(ch)).ok();
                let advance = if cell.width > 1 {
                    cell.width as u16
                } else {
                    1
                };
                cursor_x = Some(x + advance);
                x += 1;
            }
        }

        queue!(out, SetAttribute(Attribute::Reset)).ok();
        out.flush().ok();

        self.force_full = false;

        // Copy current to previous (full copy, always)
        for y in 0..h {
            for x in 0..w {
                let cell = surface.cell(x, y);
                self.previous.put(x, y, cell.ch, cell.style);
            }
        }
    }

    fn invalidate(&mut self) {
        self.force_full = true;
    }

    fn waker(&self) -> Waker {
        Waker::from_fd(self.wake_write)
    }

    fn set_cursor(&mut self, req: Option<CursorRequest>) {
        let mut out = io::stdout().lock();
        match req {
            Some(c) if c.shape != CursorShape::Hidden => {
                let seq = match c.shape {
                    CursorShape::Block => "\x1b[2 q",
                    CursorShape::Underline => "\x1b[4 q",
                    CursorShape::Bar => "\x1b[6 q",
                    CursorShape::Hidden => unreachable!(),
                };
                queue!(out, cursor::MoveTo(c.x, c.y), cursor::Show).ok();
                out.write_all(seq.as_bytes()).ok();
            }
            _ => {
                queue!(out, cursor::Hide).ok();
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
