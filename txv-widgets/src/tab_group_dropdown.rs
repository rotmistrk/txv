//! TabGroup dropdown menu — searchable tab selection overlay.

use txv_core::prelude::*;

use super::tab_group::TabGroup;

impl TabGroup {
    /// Open the dropdown menu.
    pub fn open_dropdown(&mut self) {
        self.dropdown_cursor = Some(0);
        self.dropdown_filter.clear();
        self.group.mark_dirty();
    }

    /// Whether the dropdown is currently open.
    pub fn dropdown_open(&self) -> bool {
        self.dropdown_cursor.is_some()
    }

    /// Indices of tabs matching the current filter (fuzzy).
    fn filtered_indices(&self) -> Vec<usize> {
        if self.dropdown_filter.is_empty() {
            return (0..self.titles.len()).collect();
        }
        let query = self.dropdown_filter.to_lowercase();
        (0..self.titles.len())
            .filter(|&i| fuzzy_match(&self.titles[i], &query))
            .collect()
    }

    /// Handle a key event while dropdown is open.
    pub fn handle_dropdown_key(&mut self, key: &KeyEvent) -> HandleResult {
        let Some(cursor) = self.dropdown_cursor else {
            return HandleResult::Ignored;
        };
        match key.code {
            KeyCode::Esc => {
                self.dropdown_cursor = None;
                self.dropdown_filter.clear();
                self.group.mark_dirty();
            }
            KeyCode::Enter => {
                let filtered = self.filtered_indices();
                if let Some(&idx) = filtered.get(cursor) {
                    self.set_active(idx);
                }
                self.dropdown_cursor = None;
                self.dropdown_filter.clear();
                self.group.mark_dirty();
            }
            KeyCode::Down => {
                let count = self.filtered_indices().len();
                if count > 0 {
                    self.dropdown_cursor = Some((cursor + 1) % count);
                    self.group.mark_dirty();
                }
            }
            KeyCode::Up => {
                let count = self.filtered_indices().len();
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
            KeyCode::Backspace => {
                self.dropdown_filter.pop();
                // Clamp cursor to new filtered list
                let count = self.filtered_indices().len();
                if cursor >= count && count > 0 {
                    self.dropdown_cursor = Some(count - 1);
                }
                self.group.mark_dirty();
            }
            KeyCode::Char(c) if !key.modifiers.ctrl && !key.modifiers.alt => {
                self.dropdown_filter.push(c);
                // Reset cursor to 0 on new filter input
                self.dropdown_cursor = Some(0);
                self.group.mark_dirty();
            }
            KeyCode::Char(c) if c.is_ascii_digit() && key.modifiers == KeyMod::default() => {
                // Already handled by Char branch above
            }
            _ => {}
        }
        HandleResult::Consumed
    }

    /// Move dropdown cursor down (wraps around).
    pub fn dropdown_move_down(&mut self) {
        if let Some(cursor) = self.dropdown_cursor {
            let count = self.filtered_indices().len();
            if count > 0 {
                self.dropdown_cursor = Some((cursor + 1) % count);
                self.group.mark_dirty();
            }
        }
    }

    /// Move dropdown cursor up (wraps around).
    pub fn dropdown_move_up(&mut self) {
        if let Some(cursor) = self.dropdown_cursor {
            let count = self.filtered_indices().len();
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
        let match_style = Style {
            fg: pal.interactive.search_match.fg.unwrap_or(Color::Reset),
            ..normal
        };

        let filtered = self.filtered_indices();
        let count = filtered.len().min(10);
        let max_w = filtered
            .iter()
            .take(count)
            .map(|&i| format!(" {}:{}", i, self.titles[i]).len())
            .max()
            .unwrap_or(6);
        let filter_w = if self.dropdown_filter.is_empty() {
            0
        } else {
            self.dropdown_filter.len() + 3 // "/ filter "
        };
        let dw = ((max_w.max(filter_w) + 2) as u16).min(w);
        let start_y = 1u16;
        let avail_h = h.saturating_sub(start_y + 2) as usize; // room for border + filter
        let visible = count.min(avail_h);
        let scroll = if cursor >= visible {
            cursor - visible + 1
        } else {
            0
        };

        // Filter prompt at top
        if !self.dropdown_filter.is_empty() {
            let prompt = format!("/{}", self.dropdown_filter);
            let padded = format!("{:<width$}", prompt, width = (dw - 2) as usize);
            self.group.buffer_mut().put(0, start_y, g.box_drawing.v, border);
            self.group.buffer_mut().print(1, start_y, &padded, match_style);
            if dw > 1 {
                self.group.buffer_mut().put(dw - 1, start_y, g.box_drawing.v, border);
            }
        }

        let list_y = if self.dropdown_filter.is_empty() {
            start_y
        } else {
            start_y + 1
        };

        for vi in 0..visible {
            let fi = scroll + vi;
            let row_y = list_y + vi as u16;
            if row_y >= h.saturating_sub(1) {
                break;
            }
            let tab_idx = filtered[fi];
            let title = &self.titles[tab_idx];
            let entry = format!(" {tab_idx}:{title}");
            let padded = format!("{:<width$}", entry, width = (dw - 2) as usize);
            let st = if fi == cursor {
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
        let bot_y = list_y + visible as u16;
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

/// Simple fuzzy match: all chars of query appear in order in target.
fn fuzzy_match(target: &str, query: &str) -> bool {
    let mut chars = query.chars();
    let mut next = chars.next();
    for c in target.chars().flat_map(|c| c.to_lowercase()) {
        if let Some(q) = next {
            if c == q {
                next = chars.next();
            }
        } else {
            return true;
        }
    }
    next.is_none()
}
