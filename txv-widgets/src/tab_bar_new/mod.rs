//! TabBar — horizontal tab strip with powerline separators.
//!
//! Three modes: Single (one tab shown + count), Static (all visible, fixed),
//! LRU (active leftmost, rest by recency). Supports searchable dropdown,
//! M-digit switching, transparent fill, and configurable palette.

use txv_core::prelude::*;

mod draw;
mod draw_multi;
mod dropdown;
pub mod types;

use types::SUBSCRIPTS;
pub use types::{TabBarFill, TabBarMode, TabBarPalette, TabStyle};

/// The tab bar widget.
pub struct TabBar {
    pub(crate) state: ViewState,
    pub(crate) titles: Vec<String>,
    pub(crate) dirty: Vec<bool>,
    /// Per-tab badge string (e.g. activity indicator from glyphs).
    /// None = no badge for that tab.
    pub(crate) badges: Vec<Option<String>>,
    pub(crate) active: usize,
    pub(crate) lru_order: Vec<usize>,
    pub(crate) mode: TabBarMode,
    pub(crate) palette: TabBarPalette,
    pub(crate) fill: TabBarFill,
    pub(crate) focused: bool,
    pub(crate) scroll_offset: usize,
    pub(crate) handle_keys: bool,
    /// Dropdown state: Some(cursor) when open.
    pub(crate) dropdown_cursor: Option<usize>,
    pub(crate) dropdown_filter: String,
}

impl TabBar {
    pub fn new(mode: TabBarMode) -> Self {
        Self {
            state: ViewState::new(ViewOptions {
                focusable: false,
                ..ViewOptions::default()
            }),
            titles: Vec::new(),
            dirty: Vec::new(),
            badges: Vec::new(),
            active: 0,
            lru_order: Vec::new(),
            mode,
            palette: TabBarPalette::default(),
            fill: TabBarFill::default(),
            focused: false,
            scroll_offset: 0,
            handle_keys: true,
            dropdown_cursor: None,
            dropdown_filter: String::new(),
        }
    }

    pub fn set_palette(&mut self, palette: TabBarPalette) {
        self.palette = palette;
    }

    pub fn set_fill(&mut self, fill: TabBarFill) {
        self.fill = fill;
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
        self.state.mark_dirty();
    }

    pub fn set_handle_keys(&mut self, enabled: bool) {
        self.handle_keys = enabled;
    }

    pub fn handle_keys(&self) -> bool {
        self.handle_keys
    }

    pub fn add_tab(&mut self, title: impl Into<String>) {
        self.titles.push(title.into());
        self.dirty.push(false);
        self.badges.push(None);
        self.lru_order.push(self.titles.len() - 1);
        self.state.mark_dirty();
    }

    pub fn remove_tab(&mut self, idx: usize) {
        if idx >= self.titles.len() {
            return;
        }
        self.titles.remove(idx);
        self.dirty.remove(idx);
        self.badges.remove(idx);
        self.lru_order.retain(|&i| i != idx);
        for v in &mut self.lru_order {
            if *v > idx {
                *v -= 1;
            }
        }
        if self.active >= self.titles.len() && self.active > 0 {
            self.active -= 1;
        }
        self.state.mark_dirty();
    }

    pub fn set_active(&mut self, idx: usize) {
        if idx < self.titles.len() {
            self.active = idx;
            self.touch_lru(idx);
            self.state.mark_dirty();
        }
    }

    pub fn active_index(&self) -> usize {
        self.active
    }

    pub fn tab_count(&self) -> usize {
        self.titles.len()
    }

    pub fn set_dirty(&mut self, idx: usize, is_dirty: bool) {
        if let Some(d) = self.dirty.get_mut(idx) {
            *d = is_dirty;
            self.state.mark_dirty();
        }
    }

    pub fn set_title(&mut self, idx: usize, title: impl Into<String>) {
        if let Some(t) = self.titles.get_mut(idx) {
            *t = title.into();
            self.state.mark_dirty();
        }
    }

    /// Set a badge string for a tab (e.g. activity indicator).
    /// Use `glyphs().chrome.badge_busy` etc. for standard glyphs.
    /// Pass `None` to clear.
    pub fn set_badge(&mut self, idx: usize, badge: Option<String>) {
        if let Some(b) = self.badges.get_mut(idx) {
            *b = badge;
            self.state.mark_dirty();
        }
    }

    /// Current dropdown filter text (for rendering search indicator).
    pub fn dropdown_filter(&self) -> &str {
        &self.dropdown_filter
    }

    fn touch_lru(&mut self, idx: usize) {
        self.lru_order.retain(|&i| i != idx);
        self.lru_order.insert(0, idx);
    }

    /// Get display order based on mode.
    pub(crate) fn display_order(&self) -> Vec<usize> {
        match self.mode {
            TabBarMode::Single => vec![self.active],
            TabBarMode::Static => (0..self.titles.len()).collect(),
            TabBarMode::Lru => {
                let mut order = vec![self.active];
                for &i in &self.lru_order {
                    if i != self.active && i < self.titles.len() {
                        order.push(i);
                    }
                }
                order
            }
        }
    }

    /// Number label for a tab at display position.
    pub(crate) fn number_label(&self, display_pos: usize, tab_idx: usize) -> Option<char> {
        match self.mode {
            TabBarMode::Single => None,
            TabBarMode::Static => {
                if display_pos < 9 {
                    Some(SUBSCRIPTS[display_pos + 1])
                } else {
                    None
                }
            }
            TabBarMode::Lru => {
                if tab_idx == self.active {
                    None
                } else if display_pos > 0 && display_pos <= 9 {
                    Some(SUBSCRIPTS[display_pos])
                } else {
                    None
                }
            }
        }
    }

    /// Tab style for a display position.
    pub(crate) fn tab_style(&self, display_pos: usize, tab_idx: usize) -> TabStyle {
        if tab_idx == self.active {
            if self.focused {
                self.palette.active_focused
            } else {
                self.palette.active_unfocused
            }
        } else {
            // Gradient based on distance from active tab
            let active_pos = match self.mode {
                TabBarMode::Lru => 0, // active is always first in LRU
                _ => self.active,
            };
            let distance = display_pos.abs_diff(active_pos);
            let idx = distance.saturating_sub(1).min(9);
            self.palette.inactive[idx]
        }
    }

    /// Background color of the active tab (for dropdown border matching).
    pub fn active_tab_bg(&self) -> Color {
        if self.focused {
            self.palette.active_focused.bg
        } else {
            self.palette.active_unfocused.bg
        }
    }
}

impl View for TabBar {
    delegate_view_state!(state, override { draw, handle });

    fn draw(&mut self) {
        self.draw_bar();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        self.handle_event(event)
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
