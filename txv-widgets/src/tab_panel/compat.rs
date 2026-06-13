//! Additional TabPanel APIs: compat methods, by-ID, by-title (deprecated).

use txv_core::prelude::*;

use super::tab_dropdown_source::TabDropdownSource;
use super::TabPanel;
use crate::dropdown_menu::{DropdownMenu, FilterMode, NumberMode, OpenSide};
use crate::tab_bar::TabBarMode;

impl TabPanel {
    /// Insert a tab at a specific index.
    pub fn insert_tab_at(&mut self, idx: usize, title: impl Into<String>, view: Box<dyn View>) {
        let pos = idx.min(self.tab_count());
        let gi = pos + 1; // group index
        self.bar_mut().insert_tab(pos, title);
        self.group.insert_at(gi, view);
        self.relayout();
    }

    /// Take (remove) a tab by index. Returns the view (alias for remove_tab).
    pub fn take_tab(&mut self, idx: usize) -> Option<Box<dyn View>> {
        self.remove_tab(idx)
    }

    /// Close the active tab. Returns the removed view.
    pub fn close_active(&mut self) -> Option<Box<dyn View>> {
        if self.tab_count() == 0 {
            return None;
        }
        self.remove_tab(self.bar().active_index())
    }

    /// Check if a tab can be closed.
    pub fn can_close_tab(&self, idx: usize) -> CloseResult {
        let gi = idx + 1;
        match self.group.child(gi) {
            Some(v) => v.can_close(),
            None => CloseResult::Ok,
        }
    }

    /// Index of the least recently used tab.
    pub fn lru_index(&self) -> Option<usize> {
        self.bar().lru_order().last().copied()
    }

    /// Cycle to next tab.
    pub fn tab_next(&mut self) {
        if self.tab_count() > 1 {
            let next = (self.bar().active_index() + 1) % self.tab_count();
            self.set_active(next);
        }
    }

    /// Cycle to previous tab.
    pub fn tab_prev(&mut self) {
        if self.tab_count() > 1 {
            let prev = if self.bar().active_index() == 0 {
                self.tab_count() - 1
            } else {
                self.bar().active_index() - 1
            };
            self.set_active(prev);
        }
    }

    /// Access a child view immutably by tab index.
    pub fn view_at(&self, idx: usize) -> Option<&(dyn View + '_)> {
        let gi = idx + 1;
        self.group.child(gi).map(|v| v as &dyn View)
    }

    /// Open the tab dropdown.
    pub fn open_dropdown(&mut self) {
        if self.dropdown_active {
            return;
        }
        let order = self.bar().display_order();
        let source = self.build_dropdown_source(&order);
        let number_mode = match self.bar().mode() {
            TabBarMode::Static | TabBarMode::Single => NumberMode::All,
            TabBarMode::Lru => NumberMode::SkipFirst,
        };
        let active = self.bar().active_index();
        let cursor_pos = order.iter().position(|&i| i == active).unwrap_or(0);
        let menu = DropdownMenu::new(source)
            .with_numbers(number_mode)
            .with_filter(FilterMode::Prefix)
            .with_open_side(OpenSide::Top)
            .with_border_style(palette().style(StyleId::DropdownBorder))
            .with_cursor(cursor_pos);
        let cr = self.content_rect();
        let w = self.dropdown_width().min(cr.w());
        let h = (order.len() as u16 + 1).min(cr.h());
        self.dropdown_order = order;
        self.group.insert(Box::new(menu));
        let idx = self.group.child_count() - 1;
        self.group.set_child_bounds(idx, Rect::new(cr.x() + 1, cr.y(), w, h));
        self.group.set_focused_index(idx);
        self.bar_mut().set_handle_keys(false);
        self.group.mark_dirty();
        self.dropdown_active = true;
    }

    fn build_dropdown_source(&self, order: &[usize]) -> TabDropdownSource {
        let titles: Vec<String> = order.iter().map(|&i| self.bar().titles()[i].clone()).collect();
        let dirty: Vec<bool> = order
            .iter()
            .map(|&i| self.bar().dirty_flags().get(i).copied().unwrap_or(false))
            .collect();
        let badges: Vec<Option<String>> = order
            .iter()
            .map(|&i| self.bar().badges().get(i).cloned().flatten())
            .collect();
        let badge_styles: Vec<Option<Style>> = order
            .iter()
            .map(|&i| self.bar().badge_styles().get(i).cloned().flatten())
            .collect();
        TabDropdownSource::from_parts(&titles, &dirty, &badges, &badge_styles)
    }

    /// Close the tab dropdown (remove from group).
    pub fn close_dropdown(&mut self) {
        if self.dropdown_active {
            let idx = self.group.child_count() - 1;
            self.group.remove(idx);
            let gi = self.bar().active_index() + 1;
            self.group.set_focused_index(gi);
            self.bar_mut().set_handle_keys(true);
            self.group.mark_dirty();
            self.dropdown_active = false;
        }
    }

    /// Whether the dropdown is open.
    pub fn dropdown_open(&self) -> bool {
        self.dropdown_active
    }

    // --- By-ID APIs ---

    /// Find tab index by child ViewId.
    pub fn find_tab_by_id(&self, id: ViewId) -> Option<usize> {
        for i in 0..self.tab_count() {
            let gi = i + 1;
            if let Some(child) = self.group.child(gi) {
                if child.view_id() == id {
                    return Some(i);
                }
            }
        }
        None
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
        self.view_at_mut(idx)
    }

    // --- Deprecated by-title APIs ---

    #[deprecated(note = "title is not an ID — use find_tab_by_id")]
    pub fn find_tab_by_title(&self, title: &str) -> Option<usize> {
        self.bar().titles().iter().position(|t| t == title)
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

    /// Generate next unique tab name with prefix.
    pub fn next_tab_name(&self, prefix: &str) -> String {
        let count = self.bar().titles().iter().filter(|t| t.starts_with(prefix)).count();
        format!("{prefix}:{count}")
    }

    /// Rename the active tab's user-visible part (after the colon).
    pub fn rename_user_part(&mut self, new_name: &str) {
        let idx = self.bar().active_index();
        if let Some(title) = self.bar().titles().get(idx) {
            let new_title = if let Some(colon_pos) = title.find(':') {
                format!("{}:{new_name}", &title[..colon_pos])
            } else {
                new_name.to_string()
            };
            self.bar_mut().set_title(idx, new_title);
        }
    }

    /// Sync active view's subtitle into the tab title (e.g. OSC title from PTY).
    pub(super) fn sync_subtitle(&mut self) {
        let idx = self.bar().active_index();
        let gi = idx + 1;
        let subtitle = match self.group.child(gi) {
            Some(child) => {
                let s = child.subtitle();
                if s.is_empty() {
                    return;
                }
                s.to_string()
            }
            None => return,
        };
        let current = match self.bar().titles().get(idx) {
            Some(t) => t.clone(),
            None => return,
        };
        let new_title = if let Some(colon_pos) = current.find(':') {
            let prefix = &current[..colon_pos];
            let new = format!("{prefix}:{subtitle}");
            if new == current {
                return;
            }
            new
        } else {
            return;
        };
        self.bar_mut().set_title(idx, new_title);
    }
}
