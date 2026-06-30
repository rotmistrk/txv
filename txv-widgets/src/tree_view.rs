//! TreeView — generic tree widget parameterized by TreeData.

#[path = "tree_view_draw.rs"]
mod draw;

use txv_core::prelude::*;

use crate::scroll_view::ScrollView;

/// Trait for providing tree data to TreeView.
pub trait TreeData: Send + 'static {
    fn root_count(&self) -> usize;
    fn child_count(&self, id: usize) -> usize;
    fn label(&self, id: usize) -> &str;
    fn is_expandable(&self, id: usize) -> bool;
    fn is_expanded(&self, id: usize) -> bool;
    fn toggle(&mut self, id: usize);
    fn depth(&self, id: usize) -> usize;
    /// Return flat visible row count.
    fn visible_count(&self) -> usize;
    /// Return the node id for a given visible row index.
    fn visible_id(&self, row: usize) -> usize;
    /// Style for a node (default: default style).
    fn style(&self, _id: usize) -> Style {
        Style::default()
    }
    /// Character positions to highlight in the label (for filter matches).
    fn highlight_positions(&self, _id: usize) -> Option<&[usize]> {
        None
    }
    /// Optional filter status text to show at the bottom of the tree.
    fn filter_status(&self) -> Option<&str> {
        None
    }
    /// Optional colored badge for a node (e.g. root color indicator).
    fn badge_color(&self, _id: usize) -> Option<Color> {
        None
    }
    /// Whether this file node is currently open in an editor tab.
    fn is_open(&self, _id: usize) -> bool {
        false
    }
    /// Optional icon glyph (Nerd Font) to show before label.
    fn icon(&self, _id: usize) -> Option<&str> {
        None
    }
    /// Whether node at visible index is the last sibling (for connector lines).
    /// Default: checks if next visible node has depth <= this one.
    fn is_last_sibling(&self, row: usize) -> bool {
        let depth = self.depth(self.visible_id(row));
        for i in (row + 1)..self.visible_count() {
            let d = self.depth(self.visible_id(i));
            if d < depth {
                return true;
            }
            if d == depth {
                return false;
            }
        }
        true
    }
}

pub struct TreeView<D: TreeData> {
    pub(crate) state: ViewState,
    pub(crate) data: D,
    pub(crate) cursor: usize,
    pub(crate) scroll: ScrollView,
    pub(crate) show_connectors: bool,
}

impl<D: TreeData> TreeView<D> {
    pub fn new(data: D) -> Self {
        Self {
            state: ViewState::default(),
            data,
            cursor: 0,
            scroll: ScrollView::new(),
            show_connectors: false,
        }
    }

    pub fn data(&self) -> &D {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut D {
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

    pub fn scroll_offset(&self) -> usize {
        self.scroll.offset
    }

    pub fn set_show_connectors(&mut self, show: bool) {
        self.show_connectors = show;
        self.state.mark_dirty();
    }

    pub fn state_mut(&mut self) -> &mut ViewState {
        &mut self.state
    }

    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.state.buffer_mut()
    }

    pub fn is_focused(&self) -> bool {
        self.state.is_focused()
    }

    pub fn mark_dirty(&mut self) {
        self.state.mark_dirty();
    }

    fn sync_scroll(&mut self) {
        let h = self.state.bounds().h() as usize;
        self.scroll.set_viewport(h);
        self.scroll.set_total(self.data.visible_count());
        self.scroll.ensure_visible(self.cursor);
    }

    fn clamp_cursor(&mut self) {
        let max = self.data.visible_count().saturating_sub(1);
        if self.cursor > max {
            self.cursor = max;
            self.sync_scroll();
        }
    }
}

impl<D: TreeData> View for TreeView<D> {
    delegate_view_state!(state);

    fn draw(&mut self) {
        self.clamp_cursor();
        self.draw_tree();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        match key.code() {
            KeyCode::Up => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.sync_scroll();
                    self.state.mark_dirty();
                }
                HandleResult::Consumed
            }
            KeyCode::Down => {
                let max = self.data.visible_count().saturating_sub(1);
                if self.cursor < max {
                    self.cursor += 1;
                    self.sync_scroll();
                    self.state.mark_dirty();
                }
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
            KeyCode::Home | KeyCode::End | KeyCode::PageDown | KeyCode::PageUp => {
                self.handle_jump(key.code());
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl<D: TreeData> TreeView<D> {
    fn handle_enter_right(&mut self) {
        if self.cursor >= self.data.visible_count() {
            return;
        }
        let id = self.data.visible_id(self.cursor);
        if self.data.is_expandable(id) && !self.data.is_expanded(id) {
            self.data.toggle(id);
            self.sync_scroll();
            self.state.mark_dirty();
        } else {
            self.state.put_command(CM_OK, Some(Box::new(id)));
        }
    }

    fn handle_left(&mut self) {
        if self.cursor >= self.data.visible_count() {
            return;
        }
        let id = self.data.visible_id(self.cursor);
        if self.data.is_expandable(id) && self.data.is_expanded(id) {
            self.data.toggle(id);
        } else {
            let my_depth = self.data.depth(id);
            if my_depth > 0 {
                for row in (0..self.cursor).rev() {
                    let pid = self.data.visible_id(row);
                    if self.data.depth(pid) < my_depth {
                        self.cursor = row;
                        break;
                    }
                }
            }
        }
        self.sync_scroll();
        self.state.mark_dirty();
    }

    fn handle_jump(&mut self, code: KeyCode) {
        let page = (self.state.bounds().h() as usize).saturating_sub(1).max(1);
        let max = self.data.visible_count().saturating_sub(1);
        match code {
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = max,
            KeyCode::PageDown => self.cursor = (self.cursor + page).min(max),
            KeyCode::PageUp => self.cursor = self.cursor.saturating_sub(page),
            _ => {}
        }
        self.sync_scroll();
        self.state.mark_dirty();
    }
}

#[cfg(test)]
#[path = "tree_view_sibling_tests.rs"]
mod sibling_tests;
