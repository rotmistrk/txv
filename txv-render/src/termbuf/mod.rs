//! TermBuf — VTE-driven virtual terminal emulator that renders to a Buffer.

mod resize;
mod row;
mod scrollback;
mod vte_actions;
mod vte_handler;

use txv_core::buffer::Buffer;
use txv_core::cell::Style;

use row::Row;
use scrollback::Scrollback;
use vte_handler::Performer;

/// Virtual terminal buffer backed by VTE parser.
pub struct TermBuf {
    cols: u16,
    rows: u16,
    cells: Vec<Row>,
    cursor_x: u16,
    cursor_y: u16,
    cursor_visible: bool,
    style: Style,
    reversed: bool,
    saved_cursor: (u16, u16),
    scroll_top: u16,
    scroll_bottom: u16,
    parser: vte::Parser,
    responses: Vec<Vec<u8>>,
    /// Window title set by OSC 0/2.
    osc_title: Option<String>,
    /// When true, swallow all output until ESC \ (string terminator).
    swallow_until_st: bool,
    /// Saw ESC while in swallow mode (next byte might be \).
    swallow_saw_esc: bool,
    scrollback: Scrollback,
}

#[derive(Clone)]
pub struct TCell {
    pub ch: char,
    pub style: Style,
    #[allow(dead_code)]
    pub width: u8,
}

impl Default for TCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            style: Style::default(),
            width: 1,
        }
    }
}

impl TermBuf {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self::with_scrollback(cols, rows, 2000)
    }

    pub fn with_scrollback(cols: u16, rows: u16, scrollback_limit: usize) -> Self {
        let cells = (0..rows).map(|_| Row::new(cols as usize)).collect();
        Self {
            cols,
            rows,
            cells,
            cursor_x: 0,
            cursor_y: 0,
            cursor_visible: true,
            style: Style::default(),
            reversed: false,
            saved_cursor: (0, 0),
            scroll_top: 0,
            scroll_bottom: rows.saturating_sub(1),
            parser: vte::Parser::new(),
            responses: Vec::new(),
            osc_title: None,
            swallow_until_st: false,
            swallow_saw_esc: false,
            scrollback: Scrollback::new(scrollback_limit),
        }
    }

    /// Feed bytes into the terminal emulator.
    pub fn process(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            // Swallow content of ESC k ... ESC \ (tmux title sequence)
            if self.swallow_until_st {
                if self.swallow_saw_esc {
                    self.swallow_saw_esc = false;
                    if byte == b'\\' {
                        self.swallow_until_st = false;
                    }
                } else if byte == 0x1b {
                    self.swallow_saw_esc = true;
                }
                if byte == 0x07 {
                    self.swallow_until_st = false;
                }
                continue;
            }
            let mut performer = Performer {
                cols: self.cols,
                rows: self.rows,
                cells: &mut self.cells,
                cursor_x: &mut self.cursor_x,
                cursor_y: &mut self.cursor_y,
                cursor_visible: &mut self.cursor_visible,
                style: &mut self.style,
                reversed: &mut self.reversed,
                saved_cursor: &mut self.saved_cursor,
                scroll_top: &mut self.scroll_top,
                scroll_bottom: &mut self.scroll_bottom,
                responses: &mut self.responses,
                swallow_flag: &mut self.swallow_until_st,
                osc_title: &mut self.osc_title,
                scrollback: &mut self.scrollback,
            };
            self.parser.advance(&mut performer, byte);
        }
    }

    /// Drain any pending response bytes (DA1, CPR replies).
    pub fn drain_responses(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.responses)
    }

    /// Take the window title if set by OSC 0/2.
    pub fn take_title(&mut self) -> Option<String> {
        self.osc_title.take()
    }

    /// Render terminal content to a Buffer.
    pub fn render_to(&self, buf: &mut Buffer) {
        let h = self.rows.min(buf.height());
        let w = self.cols.min(buf.width());
        for y in 0..h {
            for x in 0..w {
                let tc = &self.cells[y as usize].cells[x as usize];
                buf.put(x, y, tc.ch, tc.style);
            }
        }
    }

    /// Render terminal content to a Buffer at a given offset.
    pub fn render_at(&self, buf: &mut Buffer, ox: u16, oy: u16, w: u16, h: u16) {
        let rh = self.rows.min(h);
        let rw = self.cols.min(w);
        for y in 0..rh {
            for x in 0..rw {
                if ox + x < buf.width() && oy + y < buf.height() {
                    let tc = &self.cells[y as usize].cells[x as usize];
                    buf.put(ox + x, oy + y, tc.ch, tc.style);
                }
            }
        }
    }

    pub fn cursor(&self) -> (u16, u16) {
        (self.cursor_x, self.cursor_y)
    }
    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// Number of lines in the scrollback buffer.
    pub fn scrollback_len(&self) -> usize {
        self.scrollback.len()
    }

    /// Get a scrollback line by offset from bottom (0 = most recent).
    pub fn scrollback_line(&self, offset: usize) -> Option<&[TCell]> {
        self.scrollback.line_from_bottom(offset).map(|r| r.cells.as_slice())
    }

    /// Get a visible grid line by row index.
    pub fn grid_line(&self, row: usize) -> Option<&[TCell]> {
        self.cells.get(row).map(|r| r.cells.as_slice())
    }

    /// Number of visible rows.
    pub fn grid_rows(&self) -> u16 {
        self.rows
    }
}

#[cfg(test)]
mod tests;
