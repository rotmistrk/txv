//! Buffer — per-view cell grid. Each View owns one and draws at (0,0).
//! Groups composite children by blitting child buffers at child origins.

use crate::cell::{Cell, Style};
use crate::text::display_char_width;

/// Owned cell grid of width × height.
pub struct Buffer {
    cells: Vec<Cell>,
    width: u16,
    height: u16,
}

impl Buffer {
    pub fn new(w: u16, h: u16) -> Self {
        let len = (w as usize) * (h as usize);
        Self {
            cells: vec![Cell::default(); len],
            width: w,
            height: h,
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    /// Resize the buffer. Clears all content.
    pub fn resize(&mut self, w: u16, h: u16) {
        self.width = w;
        self.height = h;
        let len = (w as usize) * (h as usize);
        self.cells.clear();
        self.cells.resize(len, Cell::default());
    }

    /// Clear all cells to space with given style.
    pub fn fill(&mut self, ch: char, style: Style) {
        for cell in &mut self.cells {
            *cell = Cell { ch, style, width: 1 };
        }
    }

    /// Read a cell.
    pub fn cell(&self, x: u16, y: u16) -> &Cell {
        debug_assert!(x < self.width && y < self.height);
        &self.cells[self.idx(x, y)]
    }

    /// Write a character at (x, y).
    pub fn put(&mut self, x: u16, y: u16, ch: char, style: Style) {
        if x >= self.width || y >= self.height {
            return;
        }
        let cw = display_char_width(ch);
        let i = self.idx(x, y);
        self.cells[i] = Cell {
            ch,
            style,
            width: cw.max(1) as u8,
        };
        // Continuation cell for wide chars
        if cw == 2 && x + 1 < self.width {
            let j = self.idx(x + 1, y);
            self.cells[j] = Cell {
                ch: ' ',
                style,
                width: 0,
            };
        }
    }

    /// Print text starting at (x, y). Stops at buffer edge.
    pub fn print(&mut self, x: u16, y: u16, text: &str, style: Style) {
        let mut col = x;
        for ch in text.chars() {
            let cw = display_char_width(ch);
            if col + cw > self.width {
                break;
            }
            self.put(col, y, ch, style);
            col += cw;
        }
    }

    /// Print text at (x, y), fill remaining width with spaces.
    pub fn print_line(&mut self, x: u16, y: u16, text: &str, width: u16, style: Style) {
        let mut col = x;
        let end = x.saturating_add(width).min(self.width);
        for ch in text.chars() {
            let cw = display_char_width(ch);
            if col + cw > end {
                break;
            }
            self.put(col, y, ch, style);
            col += cw;
        }
        while col < end {
            self.put(col, y, ' ', style);
            col += 1;
        }
    }

    /// Print styled spans at (x, y), fill remaining width with spaces.
    pub fn print_spans_line(&mut self, x: u16, y: u16, spans: &[(&str, Style)], width: u16, fill_style: Style) {
        let mut col = x;
        let end = x.saturating_add(width).min(self.width);
        for &(text, style) in spans {
            for ch in text.chars() {
                let cw = display_char_width(ch);
                if col + cw > end {
                    break;
                }
                self.put(col, y, ch, style);
                col += cw;
            }
        }
        while col < end {
            self.put(col, y, ' ', fill_style);
            col += 1;
        }
    }

    /// Horizontal line.
    pub fn hline(&mut self, x: u16, y: u16, len: u16, ch: char, style: Style) {
        for col in x..x.saturating_add(len).min(self.width) {
            self.put(col, y, ch, style);
        }
    }

    /// Vertical line.
    pub fn vline(&mut self, x: u16, y: u16, len: u16, ch: char, style: Style) {
        for row in y..y.saturating_add(len).min(self.height) {
            self.put(x, row, ch, style);
        }
    }

    /// Blit another buffer onto this one at (dx, dy) with clipping.
    pub fn blit(&mut self, src: &Buffer, dx: u16, dy: u16) {
        use crate::cell::Color;
        let src_w = src.width.min(self.width.saturating_sub(dx));
        let src_h = src.height.min(self.height.saturating_sub(dy));
        for row in 0..src_h {
            for col in 0..src_w {
                let cell = &src.cells[src.idx(col, row)];
                // Skip fully transparent cells
                if cell.style.fg == Color::Transparent && cell.style.bg == Color::Transparent {
                    continue;
                }
                let di = self.idx(dx + col, dy + row);
                self.cells[di] = cell.clone();
            }
        }
    }

    /// Raw cell slice for backend flush.
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Mutable raw cell slice.
    pub fn cells_mut(&mut self) -> &mut [Cell] {
        &mut self.cells
    }

    fn idx(&self, x: u16, y: u16) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_and_read() {
        let mut buf = Buffer::new(10, 5);
        buf.put(3, 2, 'X', Style::default());
        assert_eq!(buf.cell(3, 2).ch, 'X');
        assert_eq!(buf.cell(0, 0).ch, ' ');
    }

    #[test]
    fn blit_clips() {
        let mut dst = Buffer::new(10, 10);
        let mut src = Buffer::new(5, 5);
        src.put(0, 0, 'A', Style::default());
        src.put(4, 4, 'B', Style::default());
        dst.blit(&src, 8, 8);
        // Only (0,0) and (1,1) of src fit at offset (8,8) in 10x10
        assert_eq!(dst.cell(8, 8).ch, 'A');
        // (4,4) at offset (8,8) = (12,12) — out of bounds, not blitted
        assert_eq!(dst.cell(9, 9).ch, ' ');
    }

    #[test]
    fn blit_skips_transparent() {
        use crate::cell::Color;
        let mut dst = Buffer::new(10, 1);
        dst.put(0, 0, '─', Style::default());
        dst.put(1, 0, '─', Style::default());
        dst.put(2, 0, '─', Style::default());

        let mut src = Buffer::new(3, 1);
        let transparent = Style {
            fg: Color::Transparent,
            bg: Color::Transparent,
            ..Style::default()
        };
        // Cell 0: transparent (should not overwrite dst)
        src.cells_mut()[0].ch = ' ';
        src.cells_mut()[0].style = transparent;
        // Cell 1: visible (should overwrite dst)
        src.put(1, 0, 'X', Style::default());
        // Cell 2: transparent
        src.cells_mut()[2].ch = ' ';
        src.cells_mut()[2].style = transparent;

        dst.blit(&src, 0, 0);
        assert_eq!(dst.cell(0, 0).ch, '─', "transparent should not overwrite");
        assert_eq!(dst.cell(1, 0).ch, 'X', "visible should overwrite");
        assert_eq!(dst.cell(2, 0).ch, '─', "transparent should not overwrite");
    }

    #[test]
    fn resize_clears() {
        let mut buf = Buffer::new(5, 5);
        buf.put(0, 0, 'X', Style::default());
        buf.resize(3, 3);
        assert_eq!(buf.cell(0, 0).ch, ' ');
        assert_eq!(buf.width(), 3);
        assert_eq!(buf.height(), 3);
    }
}
