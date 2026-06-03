//! TreeTableView draw implementation.

use txv_core::prelude::*;

use super::TreeTableView;
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
        // Calculate extra columns total width (each col + 1 separator)
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
        // Clear remaining rows
        let drawn = self
            .data
            .visible_count()
            .saturating_sub(self.scroll.offset)
            .min(tree_h as usize);
        for row in drawn..tree_h as usize {
            self.state.buffer_mut().hline(0, row as u16, w, ' ', Style::default());
        }
        if let Some(text) = &filter_text {
            let y = h - 1;
            let status_style = palette().style(StyleId::Dim);
            self.state.buffer_mut().hline(0, y, w, ' ', status_style);
            let display = format!("/{}", text);
            self.state.buffer_mut().print(0, y, &display, status_style);
        }
    }

    fn row_style(&self, idx: usize) -> Style {
        self.col_style(idx, None)
    }

    /// Style for a specific column on a row. If focused_col is set, only that column
    /// gets cursor background; others use normal style even on the cursor row.
    fn col_style(&self, idx: usize, col: Option<usize>) -> Style {
        let node_style = self.data.style(idx);
        if idx == self.cursor {
            let pal = palette();
            let use_cursor = match self.focused_col {
                Some(fc) => col == Some(fc),
                None => true,
            };
            if use_cursor {
                if self.state.is_focused() {
                    let cs = pal.style(StyleId::CursorFocused);
                    Style {
                        fg: node_style.fg,
                        bg: cs.bg,
                        attrs: cs.attrs,
                    }
                } else {
                    let cs = pal.style(StyleId::CursorUnfocused);
                    Style {
                        fg: node_style.fg,
                        bg: cs.bg,
                        attrs: node_style.attrs,
                    }
                }
            } else {
                node_style
            }
        } else {
            node_style
        }
    }

    fn draw_tree_cell(&mut self, idx: usize, y: u16, tree_w: u16) {
        let col_style = self.col_style(idx, Some(0));
        if self.data.raw_labels() {
            let label = self.data.label(idx);
            if let Some(positions) = self.data.highlight_positions(idx) {
                let sm = palette().style(StyleId::SearchMatch);
                let hl_style = Style {
                    fg: if sm.bg != Color::Reset {
                        sm.bg
                    } else {
                        sm.fg
                    },
                    bg: col_style.bg,
                    attrs: Attrs {
                        bold: true,
                        ..col_style.attrs
                    },
                };
                for (ci, ch) in label.chars().enumerate() {
                    let cx = ci as u16;
                    if cx >= tree_w {
                        break;
                    }
                    let s = if positions.contains(&ci) {
                        hl_style
                    } else {
                        col_style
                    };
                    self.state.buffer_mut().put(cx, y, ch, s);
                }
            } else {
                for (ci, ch) in label.chars().enumerate() {
                    let cx = ci as u16;
                    if cx >= tree_w {
                        break;
                    }
                    self.state.buffer_mut().put(cx, y, ch, col_style);
                }
            }
            return;
        }
        let depth = self.data.depth(idx);
        let indent = (depth * 2) as u16;
        let marker = if self.data.is_expandable(idx) {
            let g = glyphs();
            if self.data.is_expanded(idx) {
                g.tree.expanded
            } else {
                g.tree.collapsed
            }
        } else {
            "  "
        };
        let x = indent.min(tree_w.saturating_sub(1));
        self.state.buffer_mut().print(x, y, marker, col_style);
        let label = self.data.label(idx);
        let label_x = x + 2;
        if label_x >= tree_w {
            return;
        }
        if let Some(positions) = self.data.highlight_positions(idx) {
            let sm = palette().style(StyleId::SearchMatch);
            let hl_style = Style {
                fg: if sm.bg != Color::Reset {
                    sm.bg
                } else {
                    sm.fg
                },
                bg: col_style.bg,
                attrs: Attrs {
                    bold: true,
                    ..col_style.attrs
                },
            };
            for (ci, ch) in label.chars().enumerate() {
                let cx = label_x + ci as u16;
                if cx >= tree_w {
                    break;
                }
                let s = if positions.contains(&ci) {
                    hl_style
                } else {
                    col_style
                };
                self.state.buffer_mut().put(cx, y, ch, s);
            }
        } else {
            // Truncate label to fit within tree column width.
            let max_chars = (tree_w - label_x) as usize;
            for (ci, ch) in label.chars().enumerate() {
                if ci >= max_chars {
                    break;
                }
                self.state.buffer_mut().put(label_x + ci as u16, y, ch, col_style);
            }
        }
    }

    fn draw_extra_cols(&mut self, idx: usize, y: u16, tree_w: u16, total_w: u16) {
        let mut x = tree_w;
        let col_count = self.data.column_count().min(self.col_widths.len());
        let sep_style = palette().style(StyleId::Dim);
        for col in 0..col_count {
            if x >= total_w {
                break;
            }
            self.state.buffer_mut().put(x, y, '\u{2502}', sep_style);
            x += 1;
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
                    if let Some(dot_pos) = cell.find('.') {
                        dot_col.saturating_sub(dot_pos)
                    } else {
                        dot_col.saturating_sub(text_len)
                    }
                }
            };
            self.state.buffer_mut().print(x + offset as u16, y, cell, cell_style);
            x += cw as u16;
        }
    }
}
