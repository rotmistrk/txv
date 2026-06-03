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
            };
            let y = row as u16;
            self.state.buffer_mut().hline(0, y, w, ' ', style);
            let x = indent;
            self.state.buffer_mut().print(x, y, marker, style);
            let label = self.data.label(id);
            let label_x = x + 2;
            // Draw badge before label if present
            let badge_offset = if let Some(color) = self.data.badge_color(id) {
                let badge_style = Style {
                    fg: color,
                    bg: style.bg,
                    ..Style::default()
                };
                self.state.buffer_mut().put(label_x, y, '●', badge_style);
                2
            } else {
                0
            };
            let label_x = label_x + badge_offset;
            if let Some(positions) = self.data.highlight_positions(id) {
                let sm = palette().style(StyleId::SearchMatch);
                let hl_style = Style {
                    fg: if sm.bg != Color::Reset {
                        sm.bg
                    } else {
                        sm.fg
                    },
                    bg: style.bg,
                    attrs: Attrs {
                        bold: true,
                        ..style.attrs
                    },
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
            // Open-file indicator (right-aligned)
            if self.data.is_open(id) {
                let g = glyphs();
                let ind = g.tree.open_indicator;
                let ind_w = ind.chars().count() as u16;
                let ix = w.saturating_sub(ind_w + 1);
                if ix > label_x {
                    let dim = palette().style(StyleId::Dim);
                    let ind_style = Style {
                        fg: dim.fg,
                        bg: style.bg,
                        ..Style::default()
                    };
                    self.state.buffer_mut().print(ix, y, ind, ind_style);
                }
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
            let status_style = pal.style(StyleId::Dim);
            self.state.buffer_mut().hline(0, y, w, ' ', status_style);
            let display = format!("/{}", text);
            self.state.buffer_mut().print(0, y, &display, status_style);
        }
    }
}
