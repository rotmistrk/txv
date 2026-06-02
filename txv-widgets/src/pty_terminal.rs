//! PtyTerminal — a View that owns a TermBuf + PtySession.

use txv_core::event::Event;
use txv_core::prelude::*;
use txv_render::termbuf::TermBuf;

use crate::key_encode::key_to_bytes;
use crate::pty_session::PtySession;

/// Terminal view backed by a real PTY process.
pub struct PtyTerminal {
    pub(crate) state: ViewState,
    pub(crate) termbuf: TermBuf,
    pub(crate) session: Option<PtySession>,
    pub(crate) base_title: String,
    pub(crate) title: String,
    pub(crate) osc_suffix: String,
    pub(crate) prev_cols: u16,
    pub(crate) prev_rows: u16,
    pub(crate) exited: bool,
    pub(crate) scroll_offset: usize,
    /// Set when PTY produces output; cleared by `clear_output_flag()`.
    pub(crate) had_output: bool,
}

impl PtyTerminal {
    /// Returns true if the terminal received output since last `clear_output_flag()`.
    pub fn has_fresh_output(&self) -> bool {
        self.had_output
    }

    /// Clear the fresh output flag (call after reading badge state).
    pub fn clear_output_flag(&mut self) {
        self.had_output = false;
    }

    /// Write raw bytes to the PTY (for programmatic input).
    pub fn write_input(&mut self, data: &[u8]) {
        if let Some(session) = self.session.as_mut() {
            session.write(data);
        }
    }

    fn poll_and_feed(&mut self) {
        if self.exited {
            return;
        }
        let Some(session) = self.session.as_mut() else {
            return;
        };
        if let Some(data) = session.poll() {
            log::trace!("PTY data: {} bytes", data.len());
            self.termbuf.process(&data);
            self.scroll_offset = 0;
            self.had_output = true;
            self.state.mark_dirty();
        } else if !session.is_alive() {
            self.exited = true;
            self.title = format!("{} [exited]", self.base_title);
            self.session = None;
            self.state.mark_dirty();
            return;
        }
        if let Some(session) = self.session.as_mut() {
            for resp in self.termbuf.drain_responses() {
                session.write(&resp);
            }
        }
        if let Some(osc_title) = self.termbuf.take_title() {
            self.osc_suffix = osc_title;
            self.state.mark_dirty();
        }
    }
}

impl View for PtyTerminal {
    delegate_view_state!(state, override { title, subtitle, set_bounds, needs_redraw, draw, cursor });

    fn title(&self) -> &str {
        &self.title
    }

    fn subtitle(&self) -> &str {
        &self.osc_suffix
    }

    fn cursor(&self) -> Option<txv_core::cursor::CursorRequest> {
        if !self.state.is_focused() || self.exited || self.scroll_offset > 0 {
            return None;
        }
        let (cx, cy) = self.termbuf.cursor();
        let w = self.state.bounds().w;
        let h = self.state.bounds().h;
        if cx >= w || cy >= h {
            return None;
        }
        Some(txv_core::cursor::CursorRequest {
            x: cx,
            y: cy,
            shape: if self.termbuf.cursor_visible() {
                txv_core::cursor::CursorShape::Block
            } else {
                txv_core::cursor::CursorShape::Bar
            },
        })
    }

    fn needs_redraw(&self) -> bool {
        self.state.is_dirty() || self.session.is_some()
    }

    fn set_bounds(&mut self, r: Rect) {
        self.state.set_bounds(r);
        self.state.mark_dirty();
        let cols = r.w;
        let rows = r.h;
        if cols > 0 && rows > 0 && (cols != self.prev_cols || rows != self.prev_rows) {
            self.prev_cols = cols;
            self.prev_rows = rows;
            if self.session.is_some() {
                // Live PTY: simple resize, shell will redraw
                self.termbuf.resize_simple(cols, rows);
            } else {
                // Dead session: reflow for scrollback review
                self.termbuf.resize(cols, rows);
            }
            if let Some(session) = &self.session {
                session.resize(cols, rows);
            }
        }
    }

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        if self.scroll_offset == 0 {
            let rh = self.termbuf.grid_rows().min(h);
            let rw = self.prev_cols.min(w);
            for y in 0..rh {
                if let Some(line) = self.termbuf.grid_line(y as usize) {
                    for (x, tc) in line.iter().enumerate().take(rw as usize) {
                        self.state.buffer_mut().put(x as u16, y, tc.ch, tc.style);
                    }
                }
            }
            if self.termbuf.cursor_visible() {
                let (cx, cy) = self.termbuf.cursor();
                if cx < w && cy < h {
                    let cell = self.state.buffer_mut().cell(cx, cy);
                    let mut style = cell.style;
                    std::mem::swap(&mut style.fg, &mut style.bg);
                    let ch = cell.ch;
                    self.state.buffer_mut().put(cx, cy, ch, style);
                }
            }
        } else {
            self.draw_scrollback_to_buf();
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        match event {
            Event::Tick => {
                self.poll_and_feed();
                HandleResult::Ignored
            }
            Event::Paste(text) => {
                if self.exited {
                    return HandleResult::Consumed;
                }
                if let Some(session) = self.session.as_mut() {
                    session.write(b"\x1b[200~");
                    session.write(text.as_bytes());
                    session.write(b"\x1b[201~");
                }
                HandleResult::Consumed
            }
            Event::Key(key) => {
                if self.exited {
                    return HandleResult::Consumed;
                }
                if key.code == KeyCode::PageUp {
                    let max = self.termbuf.scrollback_len();
                    let page = (self.prev_rows as usize).saturating_sub(1).max(1);
                    self.scroll_offset = (self.scroll_offset + page).min(max);
                    self.state.mark_dirty();
                    return HandleResult::Consumed;
                }
                if key.code == KeyCode::PageDown {
                    let page = (self.prev_rows as usize).saturating_sub(1).max(1);
                    self.scroll_offset = self.scroll_offset.saturating_sub(page);
                    self.state.mark_dirty();
                    return HandleResult::Consumed;
                }
                if self.scroll_offset > 0 {
                    self.scroll_offset = 0;
                    self.state.mark_dirty();
                }
                if let Some(bytes) = key_to_bytes(key) {
                    if let Some(session) = self.session.as_mut() {
                        session.write(&bytes);
                    }
                    HandleResult::Consumed
                } else {
                    HandleResult::Ignored
                }
            }
            _ => HandleResult::Ignored,
        }
    }

    fn can_close(&self) -> CloseResult {
        if self.exited {
            CloseResult::Ok
        } else {
            CloseResult::Denied("process still running".to_string())
        }
    }
}
