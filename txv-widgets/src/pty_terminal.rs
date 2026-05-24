//! PtyTerminal — a View that owns a TermBuf + PtySession.

use std::path::Path;

use txv_core::event::Event;
use txv_core::prelude::*;
use txv_render::termbuf::TermBuf;

use crate::key_encode::key_to_bytes;
use crate::pty_session::PtySession;

/// Terminal view backed by a real PTY process.
pub struct PtyTerminal {
    pub(crate) state: ViewState,
    pub(crate) termbuf: TermBuf,
    session: Option<PtySession>,
    base_title: String,
    title: String,
    osc_suffix: String,
    prev_cols: u16,
    prev_rows: u16,
    exited: bool,
    pub(crate) scroll_offset: usize,
}

impl PtyTerminal {
    /// Spawn the user's default shell.
    pub fn spawn_shell(cols: u16, rows: u16) -> std::io::Result<Self> {
        Self::spawn_shell_with_scrollback(cols, rows, 2000)
    }

    /// Spawn the user's default shell with a custom scrollback limit.
    pub fn spawn_shell_with_scrollback(cols: u16, rows: u16, scrollback_limit: usize) -> std::io::Result<Self> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        let cwd = std::env::current_dir().unwrap_or_else(|_| "/".into());
        let session = PtySession::spawn(&shell, &[], &cwd, cols, rows)?;
        Ok(Self {
            state: ViewState::default(),
            termbuf: TermBuf::with_scrollback(cols, rows, scrollback_limit),
            session: Some(session),
            base_title: "Shell".into(),
            title: "Shell".into(),
            osc_suffix: String::new(),
            prev_cols: cols,
            prev_rows: rows,
            exited: false,
            scroll_offset: 0,
        })
    }

    /// Spawn a specific command.
    pub fn spawn_command(cmd: &str, args: &[&str], cwd: &Path, cols: u16, rows: u16) -> std::io::Result<Self> {
        Self::spawn_command_with_scrollback(cmd, args, cwd, cols, rows, 2000)
    }

    /// Spawn a specific command with additional environment variables.
    pub fn spawn_command_with_env(
        cmd: &str,
        args: &[&str],
        cwd: &Path,
        cols: u16,
        rows: u16,
        envs: &[(&str, &str)],
    ) -> std::io::Result<Self> {
        let session = PtySession::spawn_with_env(cmd, args, cwd, cols, rows, envs)?;
        Ok(Self {
            state: ViewState::default(),
            termbuf: TermBuf::with_scrollback(cols, rows, 2000),
            session: Some(session),
            base_title: cmd.into(),
            title: cmd.into(),
            osc_suffix: String::new(),
            prev_cols: cols,
            prev_rows: rows,
            exited: false,
            scroll_offset: 0,
        })
    }

    /// Spawn a specific command with a custom scrollback limit.
    pub fn spawn_command_with_scrollback(
        cmd: &str,
        args: &[&str],
        cwd: &Path,
        cols: u16,
        rows: u16,
        scrollback_limit: usize,
    ) -> std::io::Result<Self> {
        let session = PtySession::spawn(cmd, args, cwd, cols, rows)?;
        Ok(Self {
            state: ViewState::default(),
            termbuf: TermBuf::with_scrollback(cols, rows, scrollback_limit),
            session: Some(session),
            base_title: cmd.into(),
            title: cmd.into(),
            osc_suffix: String::new(),
            prev_cols: cols,
            prev_rows: rows,
            exited: false,
            scroll_offset: 0,
        })
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
    delegate_view_state!(state, override { title, subtitle, set_bounds, needs_redraw, draw });

    fn title(&self) -> &str {
        &self.title
    }

    fn subtitle(&self) -> &str {
        &self.osc_suffix
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
            log::debug!("PTY resize: {}x{} -> {}x{}", self.prev_cols, self.prev_rows, cols, rows);
            self.prev_cols = cols;
            self.prev_rows = rows;
            self.termbuf.resize(cols, rows);
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
            // Render grid into buffer
            let rh = self.termbuf.grid_rows().min(h);
            let rw = self.prev_cols.min(w);
            for y in 0..rh {
                if let Some(line) = self.termbuf.grid_line(y as usize) {
                    for (x, tc) in line.iter().enumerate().take(rw as usize) {
                        self.state.buffer_mut().put(x as u16, y, tc.ch, tc.style);
                    }
                }
            }
            // Draw cursor
            if self.termbuf.cursor_visible() {
                let (cx, cy) = self.termbuf.cursor();
                if cx < w && cy < h {
                    let cell = self.state.buffer_mut().cell(cx, cy);
                    let mut style = cell.style;
                    style.attrs.reverse = !style.attrs.reverse;
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
                    // Bracketed paste: app detects paste vs typed input
                    session.write(b"\x1b[200~");
                    session.write(text.as_bytes());
                    session.write(b"\x1b[201~");
                }
                HandleResult::Consumed
            }
            Event::Key(key) => {
                if self.exited {
                    return HandleResult::Consumed; // swallow keys, terminal is dead
                }
                // PgUp/PgDn for scrollback navigation
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
                // Any other key resets scroll position
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
