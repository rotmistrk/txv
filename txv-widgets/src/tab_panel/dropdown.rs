//! Dropdown overlay rendering for TabPanel.
//!
//! Draws a dropdown below the tab bar, connecting directly to the active tab
//! (no top border). Uses side borders and rounded bottom corners.
//! Border color matches active tab background for visual unity.
//! Dropdown is shifted 1 position right to align with tab content.

use txv_core::prelude::*;

use super::TabPanel;

impl TabPanel {
    /// Draw the dropdown overlay into the panel's buffer (below row 0).
    pub(crate) fn draw_dropdown_overlay(&mut self) {
        let entries = self.bar().dropdown_entries();
        if entries.is_empty() {
            return;
        }
        let cursor = self.bar().dropdown_cursor.unwrap_or(0);
        let active_bg = self.bar().active_tab_bg();
        let filter = self.bar().dropdown_filter().to_string();
        let g = glyphs();

        // Border color = active tab background for visual unity
        let border_style = Style {
            fg: active_bg,
            bg: Color::Reset,
            ..Style::default()
        };
        let pal = palette();
        let normal = Style {
            fg: pal.base.text.fg.unwrap_or(Color::Reset),
            bg: pal.base.text.bg.unwrap_or(Color::Reset),
            ..Style::default()
        };
        let selected = Style {
            fg: pal.interactive.cursor_focused.fg.unwrap_or(Color::Reset),
            bg: pal.interactive.cursor_focused.bg.unwrap_or(Color::Reset),
            ..Style::default()
        };

        let buf = self.group.buffer_mut();
        let buf_w = buf.width();
        let buf_h = buf.height();
        let max_rows = (buf_h.saturating_sub(2)) as usize;
        let visible = entries.len().min(max_rows);

        // Compute dropdown width: max label + padding + borders
        // Add 2 extra for LRU active tab alignment (no number prefix → "  " padding)
        let content_w = entries
            .iter()
            .take(visible)
            .map(|(_, label, _)| label.chars().count() + 4) // +4 for dots + padding
            .max()
            .unwrap_or(10) as u16;
        let box_w = (content_w + 2).min(buf_w.saturating_sub(1)); // +2 for borders, -1 for shift
        let inner_w = box_w.saturating_sub(2);

        // Shift dropdown 1 position to the right
        let x_off = 1u16;

        // Search filter line (shown when filter is non-empty)
        let has_filter = !filter.is_empty();
        let content_start_y = if has_filter {
            2u16
        } else {
            1u16
        };

        if has_filter {
            let y = 1u16;
            if y < buf_h {
                let filter_style = Style {
                    fg: pal.interactive.cursor_focused.fg.unwrap_or(Color::Reset),
                    bg: pal.base.text.bg.unwrap_or(Color::Reset),
                    ..Style::default()
                };
                if x_off < buf_w {
                    buf.put(x_off, y, g.box_drawing.v, border_style);
                }
                let search_text = format!(" /{} ", filter);
                let truncated: String = search_text.chars().take(inner_w as usize).collect();
                for col in 0..inner_w {
                    buf.put(x_off + 1 + col, y, ' ', filter_style);
                }
                buf.print(x_off + 1, y, &truncated, filter_style);
                let right_x = x_off + box_w - 1;
                if right_x < buf_w {
                    buf.put(right_x, y, g.box_drawing.v, border_style);
                }
            }
        }

        // Content rows
        for (row, (_, label, numbered)) in entries.iter().take(visible).enumerate() {
            let y = content_start_y + row as u16;
            if y >= buf_h {
                break;
            }
            let style = if row == cursor {
                selected
            } else {
                normal
            };

            // Left border
            if x_off < buf_w {
                buf.put(x_off, y, g.box_drawing.v, border_style);
            }

            // Content: unnumbered entries get extra padding to align with numbered ones
            let is_cursor = row == cursor;
            let dot = if is_cursor {
                '·'
            } else {
                ' '
            };
            let padded = if !numbered {
                format!("{dot}  {}", label)
            } else {
                format!("{dot}{}", label)
            };
            let truncated: String = padded.chars().take((inner_w as usize).saturating_sub(1)).collect();
            for col in 0..inner_w {
                buf.put(x_off + 1 + col, y, ' ', style);
            }
            buf.print(x_off + 1, y, &truncated, style);
            // Right dot pinned before border
            if is_cursor && inner_w > 0 {
                buf.put(x_off + inner_w, y, '·', style);
            }

            // Right border
            let right_x = x_off + box_w - 1;
            if right_x < buf_w {
                buf.put(right_x, y, g.box_drawing.v, border_style);
            }
        }

        // Bottom border with rounded corners
        let y_bot = content_start_y + visible as u16;
        if y_bot < buf_h {
            if x_off < buf_w {
                buf.put(x_off, y_bot, g.box_drawing.bl_round, border_style);
            }
            for bx in 1..box_w.saturating_sub(1) {
                let col = x_off + bx;
                if col < buf_w {
                    buf.put(col, y_bot, g.box_drawing.h, border_style);
                }
            }
            let right_x = x_off + box_w - 1;
            if right_x < buf_w {
                buf.put(right_x, y_bot, g.box_drawing.br_round, border_style);
            }
        }
    }
}
