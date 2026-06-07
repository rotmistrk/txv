//! Tree connector drawing — shared logic for ├└│ guides.

use txv_core::cell::Style;
use txv_core::palette::{palette, StyleId};
use txv_core::prelude::glyphs;

use crate::tree_table_source::TreeTableSource;

use super::TreeTableView;

impl<D: TreeTableSource> TreeTableView<D> {
    pub(super) fn draw_tree_connectors(&mut self, row: usize, depth: usize, y: u16, base: Style) {
        let g = glyphs();
        let guide_style = guide_style_with_bg(base);
        for level in 0..depth.saturating_sub(1) {
            let x = (level * 2) as u16;
            if self.ancestor_has_siblings(row, level + 1) {
                self.state.buffer_mut().put(x, y, g.tree.pipe, guide_style);
            }
        }
        let cx = ((depth - 1) * 2) as u16;
        let ch = if self.data.is_last_sibling(row) {
            g.tree.last_branch
        } else {
            g.tree.branch
        };
        self.state.buffer_mut().put(cx, y, ch, guide_style);
        self.state.buffer_mut().put(cx + 1, y, g.tree.horizontal, guide_style);
    }

    fn ancestor_has_siblings(&self, row: usize, target_depth: usize) -> bool {
        for i in (row + 1)..self.data.visible_count() {
            let d = self.data.depth(i);
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

fn guide_style_with_bg(base: Style) -> Style {
    let guide = palette().style(StyleId::TreeGuide);
    Style {
        fg: guide.fg,
        bg: base.bg,
        attrs: guide.attrs,
    }
}
