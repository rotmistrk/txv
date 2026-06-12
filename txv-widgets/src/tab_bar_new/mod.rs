//! TabBar — horizontal tab strip with powerline separators.
//!
//! Three modes: Single (one tab shown + count), Static (all visible, fixed),
//! LRU (active leftmost, rest by recency). Supports searchable dropdown,
//! M-digit switching, transparent fill, and configurable palette.

use txv_core::prelude::*;

mod badge_render_ctx;
mod display;
mod draw;
mod draw_multi;
mod draw_multi_badge;
mod draw_multi_ctx;
mod draw_multi_emit;
mod draw_multi_seg_ctx;
mod dropdown;
mod tab_bar_fill;
mod tab_bar_palette;
mod tab_style;
pub mod types;

pub use tab_bar_fill::TabBarFill;
pub use tab_bar_palette::TabBarPalette;
pub use tab_style::TabStyle;
pub use types::TabBarMode;
pub(crate) use dropdown::mac_option_digit;

/// The tab bar widget.
pub struct TabBar {
    pub(crate) state: ViewState,
    pub(crate) titles: Vec<String>,
    pub(crate) dirty: Vec<bool>,
    /// Per-tab badge string (e.g. activity indicator from glyphs).
    /// None = no badge for that tab.
    pub(crate) badges: Vec<Option<String>>,
    /// Per-tab badge style override. None = use tab's own style.
    pub(crate) badge_styles: Vec<Option<Style>>,
    pub(crate) active: usize,
    pub(crate) lru_order: Vec<usize>,
    pub(crate) mode: TabBarMode,
    pub(crate) palette: TabBarPalette,
    pub(crate) fill: TabBarFill,
    pub(crate) focused: bool,
    pub(crate) scroll_offset: usize,
    pub(crate) handle_keys: bool,
}

impl TabBar {
    pub fn new(mode: TabBarMode) -> Self {
        Self {
            state: ViewState::new(ViewOptions::default()),
            titles: Vec::new(),
            dirty: Vec::new(),
            badges: Vec::new(),
            badge_styles: Vec::new(),
            active: 0,
            lru_order: Vec::new(),
            mode,
            palette: TabBarPalette::default(),
            fill: TabBarFill::default(),
            focused: false,
            scroll_offset: 0,
            handle_keys: true,
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
        self.badge_styles.push(None);
        self.lru_order.push(self.titles.len() - 1);
        self.state.mark_dirty();
    }

    pub fn insert_tab(&mut self, pos: usize, title: impl Into<String>) {
        self.titles.insert(pos, title.into());
        self.dirty.insert(pos, false);
        self.badges.insert(pos, None);
        self.badge_styles.insert(pos, None);
        self.lru_order.push(pos);
        self.state.mark_dirty();
    }

    pub fn remove_tab(&mut self, idx: usize) {
        if idx >= self.titles.len() {
            return;
        }
        self.titles.remove(idx);
        self.dirty.remove(idx);
        self.badges.remove(idx);
        self.badge_styles.remove(idx);
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

    pub fn titles(&self) -> &[String] {
        &self.titles
    }

    pub fn dirty_flags(&self) -> &[bool] {
        &self.dirty
    }

    pub fn badges(&self) -> &[Option<String>] {
        &self.badges
    }

    pub fn badge_styles(&self) -> &[Option<Style>] {
        &self.badge_styles
    }

    pub fn lru_order(&self) -> &[usize] {
        &self.lru_order
    }

    pub fn mode(&self) -> TabBarMode {
        self.mode
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
    /// Use `glyphs().chrome.badge_busy()` etc. for standard glyphs.
    /// Pass `None` to clear.
    pub fn set_badge(&mut self, idx: usize, badge: Option<String>) {
        if let Some(b) = self.badges.get_mut(idx) {
            *b = badge;
            self.state.mark_dirty();
        }
    }

    /// Set badge with a custom style (color). Pass `None` to clear.
    pub fn set_badge_styled(&mut self, idx: usize, badge: Option<String>, style: Option<Style>) {
        let mut changed = false;
        if let Some(b) = self.badges.get_mut(idx) {
            if *b != badge {
                *b = badge;
                changed = true;
            }
        }
        if let Some(s) = self.badge_styles.get_mut(idx) {
            if *s != style {
                *s = style;
                changed = true;
            }
        }
        if changed {
            self.state.mark_dirty();
        }
    }

    fn touch_lru(&mut self, idx: usize) {
        self.lru_order.retain(|&i| i != idx);
        self.lru_order.insert(0, idx);
    }

    /// Background color of the active tab (for dropdown border matching).
    pub fn active_tab_bg(&self) -> Color {
        if self.focused {
            self.palette.active_focused.bg()
        } else {
            self.palette.active_unfocused.bg()
        }
    }
}

impl View for TabBar {
    delegate_view_state!(state, override { draw, handle, as_any_mut });

    fn draw(&mut self) {
        self.draw_bar();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        self.handle_event(event)
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
