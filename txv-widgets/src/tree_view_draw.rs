//! TreeView draw implementation.

use txv_core::prelude::*;

use super::{TreeData, TreeView};

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
            let id = self.data.visible_id(idx);
            let depth = self.data.depth(id);
            let indent = (depth * 2) as u16;
            let marker = if self.data.is_expandable(id) {
                let g = glyphs();
                if self.data.is_expanded(id) {
                    g.tree.expanded
                } else {
                    g.tree.collapsed
                }
            } else {
                "  "
            };
            let node_style = self.data.style(id);
            let style = if idx == self.cursor {
                let pal = palette();
                if self.state.is_focused() {
                    let cs = pal.interactive().cursor_focused();
                    Style {
                        fg: node_style.fg,
                        bg: cs.bg,
                        attrs: cs.attrs,
                    }
                } else {
                    let cs = pal.interactive().cursor_unfocused();
                    Style {
                        fg: node_style.fg,
                        bg: cs.bg,
                        attrs: node_style.attrs,
                    }
                }
            } else {
                node_style
            };
            let y = row as u16;
            self.state.buffer_mut().hline(0, y, w, ' ', style);
            let x = indent;
            self.state.buffer_mut().print(x, y, marker, style);
            let label = self.data.label(id);
            let label_x = x + 2;
            if let Some(positions) = self.data.highlight_positions(id) {
                let sm = palette().interactive().search_match();
                let hl_style = Style {
                    fg: if sm.fg != Color::Reset {
                        sm.fg
                    } else {
                        style.fg
                    },
                    bg: if sm.bg != Color::Reset {
                        sm.bg
                    } else {
                        style.bg
                    },
                    attrs: style.attrs,
                };
                for (ci, ch) in label.chars().enumerate() {
                    let cx = label_x + ci as u16;
                    if cx >= w {
                        break;
                    }
                    let s = if positions.contains(&ci) {
                        hl_style
                    } else {
                        style
                    };
                    self.state.buffer_mut().put(cx, y, ch, s);
                }
            } else {
                self.state.buffer_mut().print(label_x, y, label, style);
            }
        }
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
            let pal = palette();
            let status_style = pal.base().dim();
            self.state.buffer_mut().hline(0, y, w, ' ', status_style);
            let display = format!("/{}", text);
            self.state.buffer_mut().print(0, y, &display, status_style);
        }
    }
}
