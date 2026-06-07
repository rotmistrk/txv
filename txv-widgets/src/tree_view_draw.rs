//! TreeView draw implementation.

use txv_core::prelude::*;

use super::{TreeData, TreeView};
use crate::tree_draw_helpers::{draw_empty_rows, draw_filter_status, draw_highlighted_text};

impl<D: TreeData> TreeView<D> {
    pub(super) fn draw_tree(&mut self) {
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

        for row in 0..tree_h as usize {
            let idx = self.scroll.offset + row;
            if idx >= self.data.visible_count() {
                break;
            }
            self.draw_row(row, idx, w);
        }
        self.draw_empty_rows(tree_h, w);
        self.draw_filter_line(h, w, filter_text.as_deref());
    }

    fn draw_row(&mut self, row: usize, idx: usize, w: u16) {
        let id = self.data.visible_id(idx);
        let depth = self.data.depth(id);
        let indent = (depth * 2) as u16;
        let marker = if !self.data.is_expandable(id) {
            "  "
        } else {
            let g = glyphs();
            if self.data.is_expanded(id) {
                g.tree().expanded()
            } else {
                g.tree().collapsed()
            }
        };
        let node_style = self.data.style(id);
        let style = self.row_cursor_style(idx, node_style);
        let y = row as u16;
        self.state.buffer_mut().hline(0, y, w, ' ', style);
        if self.show_connectors && depth > 0 {
            self.draw_connectors(idx, depth, y, style);
        }
        self.state.buffer_mut().print(indent, y, marker, style);
        let label_x = indent + 2;
        let label_x = self.draw_badge(id, label_x, y, style);
        let label_x = self.draw_icon(id, label_x, y, node_style, style);
        self.draw_label(id, label_x, y, w, style);
        self.draw_open_indicator(id, label_x, y, w, style);
    }

    fn row_cursor_style(&self, idx: usize, node_style: Style) -> Style {
        if idx != self.cursor {
            return node_style;
        }
        let pal = palette();
        if self.state.is_focused() {
            let cs = pal.style(StyleId::CursorFocused);
            Style::new(node_style.fg(), cs.bg()).with_attrs(cs.attrs())
        } else {
            let cs = pal.style(StyleId::CursorUnfocused);
            Style::new(node_style.fg(), cs.bg()).with_attrs(node_style.attrs())
        }
    }

    fn draw_badge(&mut self, id: usize, label_x: u16, y: u16, style: Style) -> u16 {
        let Some(color) = self.data.badge_color(id) else {
            return label_x;
        };
        let badge_style = Style::new(color, style.bg());
        self.state.buffer_mut().put(label_x, y, '●', badge_style);
        label_x + 2
    }

    fn draw_icon(&mut self, id: usize, label_x: u16, y: u16, node_style: Style, style: Style) -> u16 {
        let Some(icon) = self.data.icon(id) else {
            return label_x;
        };
        let icon_style = Style::new(node_style.fg(), style.bg());
        for (i, ch) in icon.chars().enumerate() {
            self.state.buffer_mut().put(label_x + i as u16, y, ch, icon_style);
        }
        label_x + icon.chars().count() as u16
    }

    fn draw_label(&mut self, id: usize, label_x: u16, y: u16, w: u16, style: Style) {
        let label = self.data.label(id).to_string();
        let positions = self.data.highlight_positions(id).map(|p| p.to_vec());
        if let Some(positions) = positions {
            self.draw_highlighted_label(&label, &positions, label_x, y, w, style);
        } else {
            self.state.buffer_mut().print(label_x, y, &label, style);
        }
    }

    fn draw_highlighted_label(&mut self, label: &str, positions: &[usize], label_x: u16, y: u16, w: u16, style: Style) {
        draw_highlighted_text(self.state.buffer_mut(), label, positions, label_x, y, w, style);
    }

    fn draw_open_indicator(&mut self, id: usize, label_x: u16, y: u16, w: u16, style: Style) {
        if !self.data.is_open(id) {
            return;
        }
        let g = glyphs();
        let ind = g.tree().open_indicator();
        let ind_w = ind.chars().count() as u16;
        let ix = w.saturating_sub(ind_w + 1);
        if ix <= label_x {
            return;
        }
        let dim = palette().style(StyleId::Dim);
        let ind_style = Style::new(dim.fg(), style.bg());
        self.state.buffer_mut().print(ix, y, ind, ind_style);
    }

    fn draw_empty_rows(&mut self, tree_h: u16, w: u16) {
        let drawn = self
            .data
            .visible_count()
            .saturating_sub(self.scroll.offset)
            .min(tree_h as usize);
        draw_empty_rows(self.state.buffer_mut(), drawn, tree_h, w);
    }

    fn draw_filter_line(&mut self, h: u16, w: u16, filter_text: Option<&str>) {
        let Some(text) = filter_text else {
            return;
        };
        draw_filter_status(self.state.buffer_mut(), h, w, text);
    }

    fn draw_connectors(&mut self, row: usize, depth: usize, y: u16, base: Style) {
        let g = glyphs();
        let guide_style = Style::new(palette().style(StyleId::TreeGuide).fg(), base.bg());
        for level in 0..depth.saturating_sub(1) {
            let x = (level * 2) as u16;
            if self.ancestor_has_more_siblings(row, level + 1) {
                self.state.buffer_mut().put(x, y, g.tree().pipe(), guide_style);
            }
        }
        let cx = ((depth - 1) * 2) as u16;
        let ch = if self.data.is_last_sibling(row) {
            g.tree().last_branch()
        } else {
            g.tree().branch()
        };
        self.state.buffer_mut().put(cx, y, ch, guide_style);
        self.state
            .buffer_mut()
            .put(cx + 1, y, g.tree().horizontal(), guide_style);
    }

    fn ancestor_has_more_siblings(&self, row: usize, target_depth: usize) -> bool {
        for i in (row + 1)..self.data.visible_count() {
            let d = self.data.depth(self.data.visible_id(i));
            if d < target_depth {
                return false;
            }
            if d == target_depth {
                return true;
            }
        }
        false
    }
}
