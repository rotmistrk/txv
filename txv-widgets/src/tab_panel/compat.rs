//! Additional TabPanel APIs: compat methods, by-ID, by-title (deprecated).

use txv_core::prelude::*;

use super::TabPanel;

impl TabPanel {
    /// Insert a tab at a specific index.
    pub fn insert_tab_at(&mut self, idx: usize, title: impl Into<String>, view: Box<dyn View>) {
        let pos = idx.min(self.children.len());
        self.bar.titles.insert(pos, title.into());
        self.bar.dirty.insert(pos, false);
        self.bar.badges.insert(pos, None);
        self.bar.lru_order.push(pos);
        self.children.insert(pos, view);
        if let Some(sink) = self.state.sink() {
            self.children[pos].set_sink(sink.clone());
        }
        self.bar.state.mark_dirty();
        self.relayout();
    }

    /// Take (remove) a tab by index. Returns the view (alias for remove_tab).
    pub fn take_tab(&mut self, idx: usize) -> Option<Box<dyn View>> {
        self.remove_tab(idx)
    }

    /// Close the active tab. Returns the removed view.
    pub fn close_active(&mut self) -> Option<Box<dyn View>> {
        if self.children.is_empty() {
            return None;
        }
        self.remove_tab(self.bar.active_index())
    }

    /// Check if a tab can be closed.
    pub fn can_close_tab(&self, idx: usize) -> CloseResult {
        match self.children.get(idx) {
            Some(v) => v.can_close(),
            None => CloseResult::Ok,
        }
    }

    /// Index of the least recently used tab.
    pub fn lru_index(&self) -> Option<usize> {
        self.bar.lru_order.last().copied()
    }

    /// Cycle to next tab.
    pub fn tab_next(&mut self) {
        if self.children.len() > 1 {
            let next = (self.bar.active_index() + 1) % self.children.len();
            self.set_active(next);
        }
    }

    /// Cycle to previous tab.
    pub fn tab_prev(&mut self) {
        if self.children.len() > 1 {
            let prev = if self.bar.active_index() == 0 {
                self.children.len() - 1
            } else {
                self.bar.active_index() - 1
            };
            self.set_active(prev);
        }
    }

    /// Access a child view immutably by tab index.
    pub fn view_at(&self, idx: usize) -> Option<&(dyn View + '_)> {
        match self.children.get(idx) {
            Some(v) => Some(&**v),
            None => None,
        }
    }

    /// Open the tab dropdown.
    pub fn open_dropdown(&mut self) {
        self.bar.open_dropdown();
    }

    /// Close the tab dropdown.
    pub fn close_dropdown(&mut self) {
        self.bar.dropdown_cursor = None;
        self.bar.dropdown_filter.clear();
        self.bar.state.mark_dirty();
    }

    /// Whether the dropdown is open.
    pub fn dropdown_open(&self) -> bool {
        self.bar.dropdown_open()
    }

    /// Move dropdown cursor up.
    pub fn dropdown_move_up(&mut self) {
        if let Some(cursor) = self.bar.dropdown_cursor {
            let count = self.bar.dropdown_entries().len();
            if count > 0 {
                self.bar.dropdown_cursor = Some(if cursor == 0 {
                    count - 1
                } else {
                    cursor - 1
                });
                self.bar.state.mark_dirty();
            }
        }
    }

    /// Move dropdown cursor down.
    pub fn dropdown_move_down(&mut self) {
        if let Some(cursor) = self.bar.dropdown_cursor {
            let count = self.bar.dropdown_entries().len();
            if count > 0 {
                self.bar.dropdown_cursor = Some((cursor + 1) % count);
                self.bar.state.mark_dirty();
            }
        }
    }

    // --- By-ID APIs ---

    /// Find tab index by child ViewId.
    pub fn find_tab_by_id(&self, id: ViewId) -> Option<usize> {
        self.children.iter().position(|v| v.view_id() == id)
    }

    /// Close a tab by its child's ViewId.
    pub fn close_tab_by_id(&mut self, id: ViewId) -> Option<Box<dyn View>> {
        let idx = self.find_tab_by_id(id)?;
        self.remove_tab(idx)
    }

    /// Focus (activate) a tab by its child's ViewId.
    pub fn focus_tab_by_id(&mut self, id: ViewId) -> bool {
        if let Some(idx) = self.find_tab_by_id(id) {
            self.set_active(idx);
            true
        } else {
            false
        }
    }

    /// Access a tab's view by its ViewId.
    pub fn tab_view_by_id(&mut self, id: ViewId) -> Option<&mut (dyn View + '_)> {
        let idx = self.find_tab_by_id(id)?;
        match self.children.get_mut(idx) {
            Some(v) => Some(&mut **v),
            None => None,
        }
    }

    // --- Deprecated by-title APIs ---

    #[deprecated(note = "title is not an ID — use find_tab_by_id")]
    pub fn find_tab_by_title(&self, title: &str) -> Option<usize> {
        self.bar.titles.iter().position(|t| t == title)
    }

    #[deprecated(note = "title is not an ID — use close_tab_by_id")]
    pub fn close_tab_by_title(&mut self, title: &str) -> bool {
        #[allow(deprecated)]
        if let Some(idx) = self.find_tab_by_title(title) {
            self.remove_tab(idx);
            true
        } else {
            false
        }
    }

    #[deprecated(note = "title is not an ID — use focus_tab_by_id")]
    pub fn focus_tab_by_title(&mut self, title: &str) -> bool {
        #[allow(deprecated)]
        if let Some(idx) = self.find_tab_by_title(title) {
            self.set_active(idx);
            true
        } else {
            false
        }
    }

    /// Generate next unique tab name with prefix (e.g., "Shell:0", "Shell:1").
    pub fn next_tab_name(&self, prefix: &str) -> String {
        let count = self.bar.titles.iter().filter(|t| t.starts_with(prefix)).count();
        format!("{prefix}:{count}")
    }

    /// Rename the active tab's user-visible part (after the colon).
    pub fn rename_user_part(&mut self, new_name: &str) {
        let idx = self.bar.active_index();
        if let Some(title) = self.bar.titles.get(idx) {
            let new_title = if let Some(colon_pos) = title.find(':') {
                format!("{}:{new_name}", &title[..colon_pos])
            } else {
                new_name.to_string()
            };
            self.bar.set_title(idx, new_title);
        }
    }
}
