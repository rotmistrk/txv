//! TabGroup dropdown menu — tab selection overlay.

use txv_core::prelude::*;

use super::tab_group::TabGroup;

impl TabGroup {
    /// Open the dropdown menu.
    pub fn open_dropdown(&mut self) {
        self.dropdown_cursor = Some(self.group.focused_index());
        self.group.mark_dirty();
    }

    /// Whether the dropdown is currently open.
    pub fn dropdown_open(&self) -> bool {
        self.dropdown_cursor.is_some()
    }

    /// Handle a key event while dropdown is open. Returns Consumed if handled.
    pub fn handle_dropdown_key(&mut self, key: &txv_core::event::KeyEvent) -> HandleResult {
        let Some(cursor) = self.dropdown_cursor else {
            return HandleResult::Ignored;
        };
        match key.code {
            KeyCode::Esc => {
                self.dropdown_cursor = None;
                self.group.mark_dirty();
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let idx = (c as u8 - b'0') as usize;
                if idx < self.group.child_count() {
                    self.set_active(idx);
                }
                self.dropdown_cursor = None;
                self.group.mark_dirty();
            }
            KeyCode::Enter => {
                self.set_active(cursor);
                self.dropdown_cursor = None;
                self.group.mark_dirty();
            }
            KeyCode::Down => {
                let count = self.group.child_count();
                if count > 0 {
                    self.dropdown_cursor = Some((cursor + 1) % count);
                    self.group.mark_dirty();
                }
            }
            KeyCode::Up => {
                let count = self.group.child_count();
                if count > 0 {
                    let prev = if cursor == 0 {
                        count - 1
                    } else {
                        cursor - 1
                    };
                    self.dropdown_cursor = Some(prev);
                    self.group.mark_dirty();
                }
            }
            _ => {}
        }
        HandleResult::Consumed
    }

    /// Move dropdown cursor down (wraps around).
    pub fn dropdown_move_down(&mut self) {
        if let Some(cursor) = self.dropdown_cursor {
            let count = self.group.child_count();
            if count > 0 {
                self.dropdown_cursor = Some((cursor + 1) % count);
                self.group.mark_dirty();
            }
        }
    }

    /// Move dropdown cursor up (wraps around).
    pub fn dropdown_move_up(&mut self) {
        if let Some(cursor) = self.dropdown_cursor {
            let count = self.group.child_count();
            if count > 0 {
                let prev = if cursor == 0 {
                    count - 1
                } else {
                    cursor - 1
                };
                self.dropdown_cursor = Some(prev);
                self.group.mark_dirty();
            }
        }
    }

    /// Draw the dropdown overlay into own buffer.
    pub fn draw_dropdown(&mut self) {
        let Some(cursor) = self.dropdown_cursor else {
            return;
        };
        let w = self.group.buffer_mut().width();
        if w == 0 || self.titles.is_empty() {
            return;
        }
        let h = self.group.buffer_mut().height();
        let pal = palette();
        let g = glyphs();
        let border = pal.popup.border.to_style();
        let normal = pal.popup.background.to_style();
        let cursor_style = pal.popup.selected.to_style();

        let count = self.titles.len().min(10);
        let max_w = self
            .titles
            .iter()
            .enumerate()
            .map(|(i, t)| format!(" {i}:{t}").len())
            .max()
            .unwrap_or(6);
        let dw = ((max_w + 2) as u16).min(w);
        let start_y = 1u16; // Below chrome
        let avail_h = h.saturating_sub(start_y + 1) as usize;
        let visible = count.min(avail_h);
        let scroll = if cursor >= visible {
            cursor - visible + 1
        } else {
            0
        };

        for vi in 0..visible {
            let i = scroll + vi;
            let row_y = start_y + vi as u16;
            let title = self.titles.get(i).map(|s| s.as_str()).unwrap_or("");
            let entry = format!(" {i}:{title}");
            let padded = format!("{:<width$}", entry, width = (dw - 2) as usize);
            let st = if i == cursor {
                cursor_style
            } else {
                normal
            };
            self.group.buffer_mut().put(0, row_y, g.box_drawing.v, border);
            self.group.buffer_mut().print(1, row_y, &padded, st);
            if dw > 1 {
                self.group.buffer_mut().put(dw - 1, row_y, g.box_drawing.v, border);
            }
        }

        // Bottom border
        let bot_y = start_y + visible as u16;
        if bot_y < h {
            self.group.buffer_mut().put(0, bot_y, g.box_drawing.bl_round, border);
            for bx in 1..(dw - 1) {
                self.group.buffer_mut().put(bx, bot_y, g.box_drawing.h, border);
            }
            if dw > 1 {
                self.group
                    .buffer_mut()
                    .put(dw - 1, bot_y, g.box_drawing.br_round, border);
            }
        }
    }
}
