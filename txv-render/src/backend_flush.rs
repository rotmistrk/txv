//! Diff-based terminal flush for CrosstermBackend.

use std::io::{self, Write};

use crossterm::{
    cursor, queue,
    style::{self, Attribute, SetAttribute},
    terminal::{Clear, ClearType},
};
use txv_core::buffer::Buffer;
use txv_core::cell::{Cell, Style};

use crate::backend::CrosstermBackend;
use crate::color::ColorMode;
use crate::style_emit::{apply_color_mode, emit_style};

struct FlushCtx<'a> {
    buf: &'a Buffer,
    prev: &'a Buffer,
    force_full: bool,
    color_mode: ColorMode,
    last_style: Option<Style>,
}

impl CrosstermBackend {
    pub(crate) fn flush_buffer(&mut self, buf: &Buffer) {
        let (w, h) = (buf.width(), buf.height());
        if self.previous.width() != w || self.previous.height() != h {
            self.previous = Buffer::new(w, h);
            self.force_full = true;
        }

        let mut out = io::stdout().lock();
        // Hide cursor during flush to prevent flicker
        if self.last_cursor.is_some() {
            queue!(out, cursor::Hide).ok();
        }
        if self.force_full {
            queue!(out, SetAttribute(Attribute::Reset)).ok();
            queue!(out, Clear(ClearType::All)).ok();
        }

        let mut ctx = FlushCtx {
            buf,
            prev: &self.previous,
            force_full: self.force_full,
            color_mode: self.color_mode,
            last_style: None,
        };
        for y in 0..h {
            flush_row(&mut out, &mut ctx, y, w);
        }

        queue!(out, SetAttribute(Attribute::Reset)).ok();
        out.flush().ok();
        self.force_full = false;
        self.cursor_dirty = true;
        sync_previous(&mut self.previous, buf, w, h);

        // Flush images on top of text
        if !buf.images().is_empty() {
            crate::image_flush::flush_images(
                &mut io::stdout().lock(),
                buf,
                self.image_protocol,
                self.cell_size,
            );
        }
    }
}

fn flush_row(out: &mut impl Write, ctx: &mut FlushCtx<'_>, y: u16, w: u16) {
    let mut cursor_x: Option<u16> = None;
    let last_meaningful = find_last_meaningful(ctx.buf, y, w);

    let mut x = 0u16;
    while x < w {
        if x >= last_meaningful {
            clear_tail(out, ctx, y, x, w);
            break;
        }
        let cell = ctx.buf.cell(x, y);
        let pcell = ctx.prev.cell(x, y);
        if can_skip(cell, pcell, ctx.force_full) {
            x += 1;
            cursor_x = None;
            continue;
        }
        if cell.width() == 0 && pcell.width() == 0 {
            x += 1;
            continue;
        }
        if cursor_x != Some(x) {
            queue!(out, cursor::MoveTo(x, y)).ok();
        }
        cursor_x = Some(emit_cell(out, ctx, cell, x));
        x += 1;
    }
}

fn emit_cell(out: &mut impl Write, ctx: &mut FlushCtx<'_>, cell: &Cell, x: u16) -> u16 {
    let sty = apply_color_mode(cell.style(), ctx.color_mode);
    if ctx.last_style.as_ref() != Some(&sty) {
        emit_style(out, &sty);
        ctx.last_style = Some(sty);
    }
    let ch = if cell.width() == 0 {
        ' '
    } else {
        cell.ch()
    };
    queue!(out, style::Print(ch)).ok();
    let advance = if cell.width() > 1 {
        cell.width() as u16
    } else {
        1
    };
    x + advance
}

fn can_skip(cell: &Cell, prev: &Cell, force_full: bool) -> bool {
    !force_full
        && cell.ch() == prev.ch()
        && cell.style() == prev.style()
        && cell.width() == prev.width()
        && prev.width() == 1
}

fn clear_tail(out: &mut impl Write, ctx: &mut FlushCtx<'_>, y: u16, x: u16, w: u16) {
    if ctx.force_full {
        return;
    }
    let prev_had_content = (x..w).any(|px| {
        let p = ctx.prev.cell(px, y);
        p.ch() != ' ' || p.style() != Style::default() || p.width() != 1
    });
    if prev_had_content {
        queue!(out, cursor::MoveTo(x, y)).ok();
        queue!(out, SetAttribute(Attribute::Reset)).ok();
        queue!(out, Clear(ClearType::UntilNewLine)).ok();
        ctx.last_style = None;
    }
}

fn find_last_meaningful(buf: &Buffer, y: u16, w: u16) -> u16 {
    (0..w)
        .rev()
        .find(|&x| {
            let c = buf.cell(x, y);
            c.ch() != ' ' || c.style() != Style::default() || c.width() != 1
        })
        .map(|x| x + 1)
        .unwrap_or(0)
}

fn sync_previous(prev: &mut Buffer, buf: &Buffer, w: u16, h: u16) {
    for y in 0..h {
        for x in 0..w {
            let cell = buf.cell(x, y);
            prev.put(x, y, cell.ch(), cell.style());
        }
    }
}
