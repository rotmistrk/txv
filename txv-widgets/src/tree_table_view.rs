//! TreeTableView — tree with extra columns rendered on the right.

#[path = "tree_table_connectors.rs"]
mod connectors;
#[path = "tree_table_draw.rs"]
mod draw;
#[path = "tree_table_handle.rs"]
mod handle;

use txv_core::prelude::*;

use crate::scroll_view::ScrollView;
use crate::tree_table_source::TreeTableSource;

/// Tree + columns widget. Column widths are fixed; the tree column gets remaining space.
pub struct TreeTableView<D: TreeTableSource> {
    pub state: ViewState,
    pub data: D,
    pub cursor: usize,
    pub scroll: ScrollView,
    col_widths: Vec<u16>,
    focused_col: Option<usize>,
    pub(crate) h_scroll: u16,
    pub show_connectors: bool,
}

impl<D: TreeTableSource> TreeTableView<D> {
    pub fn new(data: D, col_widths: &[u16]) -> Self {
        Self {
            state: ViewState::default(),
            data,
            cursor: 0,
            scroll: ScrollView::new(),
            col_widths: col_widths.to_vec(),
            focused_col: None,
            h_scroll: 0,
            show_connectors: true,
        }
    }

    pub fn data(&self) -> &D {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut D {
        self.state.mark_dirty();
        &mut self.data
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn set_cursor(&mut self, idx: usize) {
        self.cursor = idx.min(self.data.visible_count().saturating_sub(1));
        self.sync_scroll();
        self.state.mark_dirty();
    }

    pub fn col_widths(&self) -> &[u16] {
        &self.col_widths
    }

    pub fn focused_col(&self) -> Option<usize> {
        self.focused_col
    }

    pub fn set_focused_col(&mut self, col: Option<usize>) {
        self.focused_col = col;
        self.state.mark_dirty();
    }

    pub fn set_col_widths(&mut self, widths: &[u16]) {
        self.col_widths = widths.to_vec();
        self.state.mark_dirty();
    }

    /// Compute (x_offset, width) for a given column at the current view bounds.
    /// col 0 = tree column, col 1..N = extra columns (1-indexed into col_widths).
    pub fn column_bounds(&self, col: usize) -> (u16, u16) {
        let total_w = self.state.bounds().w;
        let extra_total: u16 = self.col_widths.iter().map(|&cw| cw + 1).sum();
        let tree_w = total_w.saturating_sub(extra_total);
        if col == 0 {
            return (0, tree_w);
        }
        let idx = col - 1;
        let mut x = tree_w + 1; // skip first separator
        for i in 0..idx {
            if i < self.col_widths.len() {
                x += self.col_widths[i] + 1; // col width + next separator
            }
        }
        let w = self.col_widths.get(idx).copied().unwrap_or(0);
        (x, w)
    }

    fn sync_scroll(&mut self) {
        let h = self.state.bounds().h as usize;
        self.scroll.set_viewport(h);
        self.scroll.set_total(self.data.visible_count());
        self.scroll.ensure_visible(self.cursor);
    }

    fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.sync_scroll();
            self.state.mark_dirty();
        }
    }

    fn move_down(&mut self) {
        let max = self.data.visible_count().saturating_sub(1);
        if self.cursor < max {
            self.cursor += 1;
            self.sync_scroll();
            self.state.mark_dirty();
        }
    }

    fn page_up(&mut self) {
        let page = (self.state.bounds().h as usize).saturating_sub(1).max(1);
        self.cursor = self.cursor.saturating_sub(page);
        self.sync_scroll();
        self.state.mark_dirty();
    }

    fn page_down(&mut self) {
        let page = (self.state.bounds().h as usize).saturating_sub(1).max(1);
        let max = self.data.visible_count().saturating_sub(1);
        self.cursor = (self.cursor + page).min(max);
        self.sync_scroll();
        self.state.mark_dirty();
    }

    fn handle_enter_right(&mut self) {
        if self.cursor < self.data.visible_count()
            && self.data.is_expandable(self.cursor)
            && !self.data.is_expanded(self.cursor)
        {
            self.data.toggle(self.cursor);
            self.sync_scroll();
            self.state.mark_dirty();
        }
    }

    fn handle_left(&mut self) {
        if self.cursor >= self.data.visible_count() {
            return;
        }
        if self.data.is_expandable(self.cursor) && self.data.is_expanded(self.cursor) {
            self.data.toggle(self.cursor);
        } else {
            let my_depth = self.data.depth(self.cursor);
            if my_depth > 0 {
                for row in (0..self.cursor).rev() {
                    if self.data.depth(row) < my_depth {
                        self.cursor = row;
                        break;
                    }
                }
            }
        }
        self.sync_scroll();
        self.state.mark_dirty();
    }
}

impl<D: TreeTableSource> View for TreeTableView<D> {
    delegate_view_state!(state);

    fn draw(&mut self) {
        self.draw_tree_table();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => {
                    self.move_up();
                    HandleResult::Consumed
                }
                KeyCode::Down => {
                    self.move_down();
                    HandleResult::Consumed
                }
                KeyCode::PageUp => {
                    self.page_up();
                    HandleResult::Consumed
                }
                KeyCode::PageDown => {
                    self.page_down();
                    HandleResult::Consumed
                }
                KeyCode::Home => {
                    self.cursor = 0;
                    self.sync_scroll();
                    self.state.mark_dirty();
                    HandleResult::Consumed
                }
                KeyCode::End => {
                    self.cursor = self.data.visible_count().saturating_sub(1);
                    self.sync_scroll();
                    self.state.mark_dirty();
                    HandleResult::Consumed
                }
                KeyCode::Enter | KeyCode::Right => {
                    self.handle_enter_right();
                    HandleResult::Consumed
                }
                KeyCode::Left => {
                    self.handle_left();
                    HandleResult::Consumed
                }
                _ => self.handle_char_key(key),
            },
            _ => HandleResult::Ignored,
        }
    }
}
