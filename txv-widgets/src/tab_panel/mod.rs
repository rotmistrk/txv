//! TabPanel — a self-contained tabbed container.
//!
//! Combines a [`TabBar`] (preprocess child at index 0) with a stack of
//! content Views (one visible at a time). Uses GroupState for three-phase
//! dispatch: bar intercepts tab keys in preprocess, active content handles
//! the rest in phase 2.

use txv_core::prelude::*;

use crate::tab_bar::{TabBar, TabBarMode};

mod compat;
mod tab_dropdown_source;
mod tab_entry;
mod view_impl;

/// A tabbed panel: TabBar on top, stacked children below.
///
/// Internal layout: group child 0 = TabBar (preprocess),
/// group children 1..N = tab content views.
/// Focused index = active_tab + 1.
pub struct TabPanel {
    group: GroupState,
    dropdown_active: bool,
    /// Maps dropdown display index → real tab index (set when dropdown opens).
    dropdown_order: Vec<usize>,
}

impl TabPanel {
    pub fn new(mode: TabBarMode) -> Self {
        let mut group = GroupState::new(ViewOptions::default().with_focusable());
        let mut bar = TabBar::new(mode);
        bar.state.set_preprocess(true);
        group.insert(Box::new(bar));
        // No content children yet; focused stays at 0 (bar) until a tab is added
        Self {
            group,
            dropdown_active: false,
            dropdown_order: Vec::new(),
        }
    }

    pub fn bar(&self) -> &TabBar {
        self.group
            .child(0)
            .and_then(|c| c.as_any())
            .and_then(|a| a.downcast_ref::<TabBar>())
            .expect("child 0 is TabBar")
    }

    pub fn bar_mut(&mut self) -> &mut TabBar {
        self.group
            .child_mut(0)
            .and_then(|c| c.as_any_mut())
            .and_then(|a| a.downcast_mut::<TabBar>())
            .expect("child 0 is TabBar")
    }

    /// Insert a tab with a title and child view. Activates the new tab.
    pub fn insert_tab(&mut self, title: impl Into<String>, view: Box<dyn View>) {
        self.bar_mut().add_tab(title);
        self.group.insert(view);
        let new_idx = self.group.child_count() - 1;
        // New tab starts hidden; set_active will make it visible
        self.group.set_child_visible(new_idx, false);
        self.set_active(new_idx - 1);
    }

    /// Remove a tab by index. Returns the removed view.
    pub fn remove_tab(&mut self, idx: usize) -> Option<Box<dyn View>> {
        let gi = idx + 1; // group index
        if gi >= self.group.child_count() {
            return None;
        }
        self.bar_mut().remove_tab(idx);
        let view = self.group.remove(gi);
        self.relayout();
        Some(view)
    }

    /// Take the active tab's view (for moving between panels).
    pub fn take_active(&mut self) -> Option<(String, Box<dyn View>)> {
        let idx = self.bar().active_index();
        if idx >= self.tab_count() {
            return None;
        }
        let title = self.bar().titles()[idx].clone();
        self.bar_mut().remove_tab(idx);
        let view = self.group.remove(idx + 1);
        self.relayout();
        Some((title, view))
    }

    /// Set active tab by index.
    pub fn set_active(&mut self, idx: usize) {
        let gi = idx + 1;
        if gi >= self.group.child_count() {
            return;
        }
        let prev = self.bar().active_index();
        if prev != idx {
            let prev_gi = prev + 1;
            if prev_gi < self.group.child_count() {
                if let Some(child) = self.group.child_mut(prev_gi) {
                    child.unselect();
                }
                self.group.set_child_visible(prev_gi, false);
            }
        }
        self.group.set_child_visible(gi, true);
        self.bar_mut().set_active(idx);
        self.group.set_focused_index(gi);
        if self.group.is_focused() {
            if let Some(child) = self.group.child_mut(gi) {
                child.select();
            }
        }
        self.relayout();
    }

    /// Activate tab by its label position (₁=1, ₂=2, etc).
    pub fn activate_by_label(&mut self, label_pos: usize) {
        let prev = self.bar().active_index();
        self.bar_mut().activate_by_number(label_pos);
        let new = self.bar().active_index();
        if prev != new {
            self.sync_focus_from_bar(prev);
        }
    }

    /// Activate tab by M-digit number.
    pub fn activate_by_number(&mut self, n: usize) {
        let prev = self.bar().active_index();
        self.bar_mut().activate_by_number(n);
        let new = self.bar().active_index();
        if new != prev {
            self.sync_focus_from_bar(prev);
        }
    }

    /// Active tab index.
    pub fn active_index(&self) -> usize {
        self.bar().active_index()
    }

    /// Number of tabs.
    pub fn tab_count(&self) -> usize {
        let extra = u8::from(self.dropdown_active) as usize;
        self.group.child_count().saturating_sub(1 + extra) // exclude bar + dropdown
    }

    /// Set dirty flag on a tab.
    pub fn set_dirty(&mut self, idx: usize, dirty: bool) {
        self.bar_mut().set_dirty(idx, dirty);
    }

    /// Set badge with a custom style on a tab.
    pub fn set_badge_styled(&mut self, idx: usize, badge: Option<String>, style: Option<Style>) {
        self.bar_mut().set_badge_styled(idx, badge, style);
    }

    /// Set title of a tab.
    pub fn set_title(&mut self, idx: usize, title: impl Into<String>) {
        self.bar_mut().set_title(idx, title);
    }

    /// Set whether this panel is focused.
    pub fn set_focused(&mut self, focused: bool) {
        self.bar_mut().set_focused(focused);
    }

    /// Access active child view.
    pub fn active_child(&self) -> Option<&dyn View> {
        let gi = self.bar().active_index() + 1;
        self.group.child(gi)
    }

    /// Get origin of the active child in panel-local coordinates.
    pub fn active_child_origin(&self) -> (u16, u16) {
        let gi = self.bar().active_index() + 1;
        self.group.child_origin(gi)
    }

    /// Access active child view mutably (Box).
    pub fn active_child_mut(&mut self) -> Option<&mut Box<dyn View>> {
        let gi = self.bar().active_index() + 1;
        self.group.child_mut(gi)
    }

    /// Access active child view mutably (dyn View).
    pub fn active_view_mut(&mut self) -> Option<&mut (dyn View + '_)> {
        let gi = self.bar().active_index() + 1;
        self.group.child_mut(gi).map(|v| &mut **v as &mut dyn View)
    }

    /// Access a child view mutably by tab index.
    pub fn view_at_mut(&mut self, idx: usize) -> Option<&mut (dyn View + '_)> {
        let gi = idx + 1;
        self.group.child_mut(gi).map(|v| &mut **v as &mut dyn View)
    }

    /// Title of the active tab.
    pub fn active_title(&self) -> Option<&str> {
        let idx = self.bar().active_index();
        self.bar().titles().get(idx).map(|s| s.as_str())
    }

    /// Tab title by index.
    pub fn tab_title(&self, idx: usize) -> Option<&str> {
        self.bar().titles().get(idx).map(|s| s.as_str())
    }

    pub(crate) fn content_rect(&self) -> Rect {
        let b = self.group.bounds();
        if b.h() <= 1 {
            return Rect::new(b.x(), b.y(), b.w(), 0);
        }
        Rect::new(b.x(), b.y() + 1, b.w(), b.h() - 1)
    }

    fn dropdown_width(&self) -> u16 {
        let max_title = self.bar().titles().iter().map(|t| t.chars().count()).max().unwrap_or(4);
        let max_badge_w = self
            .bar()
            .badges()
            .iter()
            .filter_map(|b| b.as_ref())
            .map(|b| b.chars().count())
            .max()
            .unwrap_or(0);
        let badge_w: usize = if max_badge_w > 0 || self.bar().dirty_flags().iter().any(|d| *d) {
            max_badge_w.max(1) + 1
        } else {
            0
        };
        (max_title + 3 + badge_w) as u16 + 2
    }

    pub(crate) fn relayout(&mut self) {
        let b = self.group.bounds();
        if b.w() == 0 || b.h() == 0 {
            return;
        }
        // Bar gets row 0
        self.group.set_child_bounds(0, Rect::new(b.x(), b.y(), b.w(), 1));
        // Active child gets content rect and visibility, others get zero
        let cr = self.content_rect();
        let active = self.bar().active_index();
        for i in 0..self.tab_count() {
            let gi = i + 1;
            if i == active {
                self.group.set_child_bounds(gi, cr);
                self.group.set_child_visible(gi, true);
            } else {
                self.group.set_child_bounds(gi, Rect::default());
                self.group.set_child_visible(gi, false);
            }
        }
        // Reposition dropdown if active
        if self.dropdown_active {
            let dd_idx = self.group.child_count() - 1;
            let w = self.dropdown_width().min(cr.w());
            let h = (self.tab_count() as u16 + 1).min(cr.h());
            self.group.set_child_bounds(dd_idx, Rect::new(cr.x() + 1, cr.y(), w, h));
        }
        self.group.mark_dirty();
    }

    /// Sync group focus after bar changes active tab.
    fn sync_focus_from_bar(&mut self, prev: usize) {
        let new = self.bar().active_index();
        let prev_gi = prev + 1;
        if prev_gi < self.group.child_count() {
            if let Some(child) = self.group.child_mut(prev_gi) {
                child.unselect();
            }
            self.group.set_child_visible(prev_gi, false);
        }
        let new_gi = new + 1;
        self.group.set_child_visible(new_gi, true);
        self.group.set_focused_index(new_gi);
        if self.group.is_focused() {
            if let Some(child) = self.group.child_mut(new_gi) {
                child.select();
            }
        }
        self.relayout();
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
