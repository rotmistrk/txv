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
        if row + 1 >= self.visible_count() {
            return true;
        }
        self.depth(self.visible_id(row + 1)) <= depth
    }
}

pub struct TreeView<D: TreeData> {
    pub state: ViewState,
    pub data: D,
    pub cursor: usize,
    pub scroll: ScrollView,
    pub show_connectors: bool,
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
        let h = self.state.bounds().h as usize;
        self.scroll.set_viewport(h);
        self.scroll.set_total(self.data.visible_count());
        self.scroll.ensure_visible(self.cursor);
    }
}

impl<D: TreeData> View for TreeView<D> {
    delegate_view_state!(state);

    fn draw(&mut self) {
        self.draw_tree();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        match event {
            Event::Key(key) => match key.code {
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
                    if self.cursor < self.data.visible_count() {
                        let id = self.data.visible_id(self.cursor);
                        if self.data.is_expandable(id) && !self.data.is_expanded(id) {
                            self.data.toggle(id);
                            self.sync_scroll();
                            self.state.mark_dirty();
                        } else {
                            self.state.put_command(CM_OK, Some(Box::new(id)));
                        }
                    }
                    HandleResult::Consumed
                }
                KeyCode::Left => {
                    if self.cursor < self.data.visible_count() {
                        let id = self.data.visible_id(self.cursor);
                        if self.data.is_expandable(id) && self.data.is_expanded(id) {
                            // Collapse expanded directory
                            self.data.toggle(id);
                        } else {
                            // Go to parent: find nearest visible row above with depth-1
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
                KeyCode::PageDown => {
                    let page = (self.state.bounds().h as usize).saturating_sub(1).max(1);
                    let max = self.data.visible_count().saturating_sub(1);
                    self.cursor = (self.cursor + page).min(max);
                    self.sync_scroll();
                    self.state.mark_dirty();
                    HandleResult::Consumed
                }
                KeyCode::PageUp => {
                    let page = (self.state.bounds().h as usize).saturating_sub(1).max(1);
                    self.cursor = self.cursor.saturating_sub(page);
                    self.sync_scroll();
                    self.state.mark_dirty();
                    HandleResult::Consumed
                }
                _ => HandleResult::Ignored,
            },
            _ => HandleResult::Ignored,
        }
    }
}
