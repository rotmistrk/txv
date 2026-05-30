//! Diff-based terminal flush for CrosstermBackend.

use std::io::{self, Write};

use crossterm::{
    cursor, queue,
    style::{self, Attribute, SetAttribute},
    terminal::{Clear, ClearType},
};
use txv_core::buffer::Buffer;
use txv_core::cell::Style;

use crate::backend::CrosstermBackend;
use crate::style_emit::{apply_color_mode, emit_style};

impl CrosstermBackend {
    pub(crate) fn flush_buffer(&mut self, buf: &Buffer) {
        let w = buf.width();
        let h = buf.height();

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

            let last_meaningful = (0..w)
                .rev()
                .find(|&x| {
                    let c = buf.cell(x, y);
                    c.ch != ' ' || c.style != Style::default() || c.width != 1
                })
                .map(|x| x + 1)
                .unwrap_or(0);

            let mut x = 0u16;
            while x < w {
                let cell = buf.cell(x, y);
                let prev = self.previous.cell(x, y);

                if x >= last_meaningful {
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

                if cell.width == 0 && prev.width == 0 {
                    x += 1;
                    continue;
                }

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
        self.cursor_dirty = true;

        for y in 0..h {
            for x in 0..w {
                let cell = buf.cell(x, y);
                self.previous.put(x, y, cell.ch, cell.style);
            }
        }
    }
}
