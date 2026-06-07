//! TreeTableView draw implementation.

use txv_core::prelude::*;

use super::TreeTableView;
use crate::tree_draw_helpers::{draw_empty_rows, draw_filter_status, draw_highlighted_text};
use crate::tree_table_source::{ColAlign, TreeTableSource};

impl<D: TreeTableSource> TreeTableView<D> {
    pub(super) fn draw_tree_table(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let filter_text = self.data.filter_status().map(|s| s.to_string());
        let tree_h = if filter_text.is_some() {
            h.saturating_sub(1)
        } else {
            h
        };
        let extra_total: u16 = self.col_widths.iter().map(|&cw| cw + 1).sum();
        let tree_w = w.saturating_sub(extra_total);

        for row in 0..tree_h as usize {
            let idx = self.scroll.offset + row;
            if idx >= self.data.visible_count() {
                break;
            }
            let y = row as u16;
            let bg_style = if self.focused_col.is_some() && idx == self.cursor {
                Style::default()
            } else {
                self.row_style(idx)
            };
            self.state.buffer_mut().hline(0, y, w, ' ', bg_style);
            self.draw_tree_cell(idx, y, tree_w);
            self.draw_extra_cols(idx, y, tree_w, w);
        }
        self.draw_empty_rows(tree_h, w);
        self.draw_filter_status(h, w, filter_text.as_deref());
    }

    fn draw_empty_rows(&mut self, tree_h: u16, w: u16) {
        let drawn = self
            .data
            .visible_count()
            .saturating_sub(self.scroll.offset)
            .min(tree_h as usize);
        draw_empty_rows(self.state.buffer_mut(), drawn, tree_h, w);
    }

    fn draw_filter_status(&mut self, h: u16, w: u16, text: Option<&str>) {
        let Some(text) = text else {
            return;
        };
        draw_filter_status(self.state.buffer_mut(), h, w, text);
    }

    fn row_style(&self, idx: usize) -> Style {
        self.col_style(idx, None)
    }

    fn col_style(&self, idx: usize, col: Option<usize>) -> Style {
        let node_style = self.data.style(idx);
        if idx != self.cursor {
            return node_style;
        }
        let pal = palette();
        let is_cursor_col = match self.focused_col {
            Some(fc) => col == Some(fc),
            None => true,
        };
        if is_cursor_col && self.state.is_focused() {
            node_style.with_bg(pal.style(StyleId::CursorFocused).bg())
        } else {
            node_style.with_bg(pal.style(StyleId::CursorUnfocused).bg())
        }
    }

    fn draw_tree_cell(&mut self, idx: usize, y: u16, tree_w: u16) {
        let col_style = self.col_style(idx, Some(0));
        if self.data.raw_labels() {
            self.draw_raw_label(idx, y, tree_w, col_style);
            return;
        }
        self.draw_structured_tree_cell(idx, y, tree_w, col_style);
    }

    fn draw_raw_label(&mut self, idx: usize, y: u16, tree_w: u16, col_style: Style) {
        let label = self.data.label(idx).to_string();
        let positions = self.data.highlight_positions(idx).map(|p| p.to_vec());
        if let Some(positions) = positions {
            self.draw_hl_chars(&label, &positions, 0, y, tree_w, col_style);
        } else {
            for (ci, ch) in label.chars().enumerate() {
                let cx = ci as u16;
                if cx >= tree_w {
                    break;
                }
                self.state.buffer_mut().put(cx, y, ch, col_style);
            }
        }
    }

    fn draw_structured_tree_cell(&mut self, idx: usize, y: u16, tree_w: u16, col_style: Style) {
        let depth = self.data.depth(idx);
        let indent = (depth * 2) as u16;
        let marker = if self.data.is_expandable(idx) {
            let g = glyphs();
            if self.data.is_expanded(idx) {
                g.tree().expanded()
            } else {
                g.tree().collapsed()
            }
        } else {
            "  "
        };
        let x = indent.min(tree_w.saturating_sub(1));
        if self.show_connectors && depth > 0 {
            self.draw_tree_connectors(idx, depth, y, col_style);
        }
        self.state.buffer_mut().print(x, y, marker, col_style);
        let label = self.data.label(idx).to_string();
        let label_x = x + 2;
        if label_x >= tree_w {
            return;
        }
        let positions = self.data.highlight_positions(idx).map(|p| p.to_vec());
        if let Some(positions) = positions {
            self.draw_hl_chars(&label, &positions, label_x, y, tree_w, col_style);
        } else {
            let max_chars = (tree_w - label_x) as usize;
            for (ci, ch) in label.chars().enumerate() {
                if ci >= max_chars {
                    break;
                }
                self.state.buffer_mut().put(label_x + ci as u16, y, ch, col_style);
            }
        }
    }

    fn draw_hl_chars(&mut self, label: &str, positions: &[usize], start_x: u16, y: u16, max_x: u16, col_style: Style) {
        draw_highlighted_text(self.state.buffer_mut(), label, positions, start_x, y, max_x, col_style);
    }

    fn draw_extra_cols(&mut self, idx: usize, y: u16, tree_w: u16, total_w: u16) {
        let col_count = self.data.column_count().min(self.col_widths.len());
        let sep_style = palette().style(StyleId::Dim);
        let mut abs_x: u16 = 0;
        for col in 0..col_count {
            let col_total = 1 + self.col_widths[col];
            if abs_x + col_total <= self.h_scroll {
                abs_x += col_total;
                continue;
            }
            let screen_x = tree_w + abs_x.saturating_sub(self.h_scroll);
            if screen_x >= total_w {
                break;
            }
            self.draw_single_col(idx, y, col, screen_x, total_w, sep_style);
            abs_x += col_total;
        }
    }

    fn draw_single_col(&mut self, idx: usize, y: u16, col: usize, screen_x: u16, total_w: u16, sep_style: Style) {
        self.state.buffer_mut().put(screen_x, y, '\u{2502}', sep_style);
        let content_x = screen_x + 1;
        if content_x >= total_w {
            return;
        }
        let cw = self.col_widths[col] as usize;
        let cell = self.data.cell(idx, col);
        let cell_style = self.col_style(idx, Some(col + 1));
        let text_len = cell.chars().count().min(cw);
        let offset = match self.data.column_align(col) {
            ColAlign::Left => 0,
            ColAlign::Right => cw.saturating_sub(text_len),
            ColAlign::Center => cw.saturating_sub(text_len) / 2,
            ColAlign::Decimal => {
                let dot_col = cw * 2 / 3;
                cell.find('.')
                    .map_or(dot_col.saturating_sub(text_len), |dp| dot_col.saturating_sub(dp))
            }
        };
        let avail = (total_w - content_x) as usize;
        let print_text: String = cell.chars().take(avail).collect();
        self.state
            .buffer_mut()
            .print(content_x + offset as u16, y, &print_text, cell_style);
    }
}
