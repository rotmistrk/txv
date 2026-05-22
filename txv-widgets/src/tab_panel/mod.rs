//! TabPanel — a self-contained tabbed container.
//!
//! Combines a [`TabBar`] (row 0) with a stack of child Views (one visible
//! at a time, filling the remaining space). Works standalone or inside
//! TiledWorkspace.

use txv_core::prelude::*;

use crate::tab_bar::{TabBar, TabBarMode};

mod compat;
mod dropdown;

/// A tabbed panel: TabBar on top, stacked children below.
pub struct TabPanel {
    state: ViewState,
    pub(crate) bar: TabBar,
    pub(crate) children: Vec<Box<dyn View>>,
}

impl TabPanel {
    pub fn new(mode: TabBarMode) -> Self {
        Self {
            state: ViewState::new(ViewOptions {
                focusable: true,
                ..ViewOptions::default()
            }),
            bar: TabBar::new(mode),
            children: Vec::new(),
        }
    }

    pub fn bar(&self) -> &TabBar {
        &self.bar
    }

    pub fn bar_mut(&mut self) -> &mut TabBar {
        &mut self.bar
    }

    /// Insert a tab with a title and child view. Activates the new tab.
    pub fn insert_tab(&mut self, title: impl Into<String>, view: Box<dyn View>) {
        let new_idx = self.children.len();
        self.bar.add_tab(title);
        self.children.push(view);
        if let Some(sink) = self.state.sink() {
            self.children[new_idx].set_sink(sink.clone());
        }
        self.set_active(new_idx);
    }

    /// Remove a tab by index. Returns the removed view.
    pub fn remove_tab(&mut self, idx: usize) -> Option<Box<dyn View>> {
        if idx >= self.children.len() {
            return None;
        }
        self.bar.remove_tab(idx);
        let view = self.children.remove(idx);
        self.relayout();
        Some(view)
    }

    /// Take the active tab's view (for moving between panels).
    pub fn take_active(&mut self) -> Option<(String, Box<dyn View>)> {
        let idx = self.bar.active_index();
        if idx >= self.children.len() {
            return None;
        }
        let title = self.bar.titles[idx].clone();
        self.bar.remove_tab(idx);
        let view = self.children.remove(idx);
        self.relayout();
        Some((title, view))
    }

    /// Set active tab by index.
    pub fn set_active(&mut self, idx: usize) {
        let prev = self.bar.active_index();
        if prev != idx {
            if let Some(child) = self.children.get_mut(prev) {
                child.unselect();
            }
        }
        self.bar.set_active(idx);
        if self.state.is_focused() {
            if let Some(child) = self.children.get_mut(idx) {
                child.select();
            }
        }
        self.relayout();
    }

    /// Active tab index.
    pub fn active_index(&self) -> usize {
        self.bar.active_index()
    }

    /// Number of tabs.
    pub fn tab_count(&self) -> usize {
        self.children.len()
    }

    /// Set dirty flag on a tab.
    pub fn set_dirty(&mut self, idx: usize, dirty: bool) {
        self.bar.set_dirty(idx, dirty);
    }

    /// Set title of a tab.
    pub fn set_title(&mut self, idx: usize, title: impl Into<String>) {
        self.bar.set_title(idx, title);
    }

    /// Set whether this panel is focused.
    pub fn set_focused(&mut self, focused: bool) {
        self.bar.set_focused(focused);
    }

    /// Access active child view.
    pub fn active_child(&self) -> Option<&dyn View> {
        self.children.get(self.bar.active_index()).map(|v| &**v)
    }

    /// Access active child view mutably (Box).
    pub fn active_child_mut(&mut self) -> Option<&mut Box<dyn View>> {
        let idx = self.bar.active_index();
        self.children.get_mut(idx)
    }

    /// Access active child view mutably (dyn View).
    pub fn active_view_mut(&mut self) -> Option<&mut (dyn View + '_)> {
        let idx = self.bar.active_index();
        match self.children.get_mut(idx) {
            Some(v) => Some(&mut **v),
            None => None,
        }
    }

    /// Access a child view mutably by tab index.
    pub fn view_at_mut(&mut self, idx: usize) -> Option<&mut (dyn View + '_)> {
        match self.children.get_mut(idx) {
            Some(v) => Some(&mut **v),
            None => None,
        }
    }

    /// Title of the active tab.
    pub fn active_title(&self) -> Option<&str> {
        let idx = self.bar.active_index();
        self.bar.titles.get(idx).map(|s| s.as_str())
    }

    /// Tab title by index.
    pub fn tab_title(&self, idx: usize) -> Option<&str> {
        self.bar.titles.get(idx).map(|s| s.as_str())
    }

    pub(crate) fn content_rect(&self) -> Rect {
        let b = self.state.bounds();
        if b.h <= 1 {
            return Rect::new(b.x, b.y, b.w, 0);
        }
        Rect::new(b.x, b.y + 1, b.w, b.h - 1)
    }

    pub(crate) fn relayout(&mut self) {
        let b = self.state.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        self.bar.set_bounds(Rect::new(b.x, b.y, b.w, 1));
        let cr = self.content_rect();
        let active = self.bar.active_index();
        for (i, child) in self.children.iter_mut().enumerate() {
            if i == active {
                child.set_bounds(cr);
            } else {
                child.set_bounds(Rect::default());
            }
        }
        self.state.mark_dirty();
    }
}

impl View for TabPanel {
    delegate_view_state!(state, override { set_bounds, set_sink, draw, handle, select, unselect });

    fn set_bounds(&mut self, r: Rect) {
        self.state.set_bounds(r);
        self.relayout();
    }

    fn set_sink(&mut self, sink: EventSink) {
        self.state.set_sink(sink.clone());
        self.bar.set_sink(sink.clone());
        for child in &mut self.children {
            child.set_sink(sink.clone());
        }
    }

    fn select(&mut self) {
        self.state.set_focused(true);
        self.state.mark_dirty();
        if let Some(child) = self.children.get_mut(self.bar.active_index()) {
            child.select();
        }
    }

    fn unselect(&mut self) {
        self.state.set_focused(false);
        self.state.mark_dirty();
        if let Some(child) = self.children.get_mut(self.bar.active_index()) {
            child.unselect();
        }
    }

    fn draw(&mut self) {
        let b = self.state.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        self.state.buffer_mut().fill(' ', Style::default());
        self.bar.draw();
        let bar_buf = self.bar.buffer();
        let buf_ptr = self.state.buffer_mut() as *mut Buffer;
        unsafe { (*buf_ptr).blit(bar_buf, 0, 0) };

        let active = self.bar.active_index();
        if let Some(child) = self.children.get_mut(active) {
            child.draw();
            let cb = child.bounds();
            if cb.w > 0 && cb.h > 0 {
                let dx = cb.x.saturating_sub(b.x);
                let dy = cb.y.saturating_sub(b.y);
                unsafe { (*buf_ptr).blit(child.buffer(), dx, dy) };
            }
        }

        // Draw dropdown overlay on top of content
        if self.bar.dropdown_open() {
            self.draw_dropdown_overlay();
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if matches!(event, Event::Tick) {
            for child in &mut self.children {
                child.handle(event);
            }
            return HandleResult::Ignored;
        }
        let prev_active = self.bar.active_index();
        let result = self.bar.handle(event);
        if result == HandleResult::Consumed {
            if self.bar.active_index() != prev_active {
                self.relayout();
            }
            return HandleResult::Consumed;
        }
        let active = self.bar.active_index();
        if let Some(child) = self.children.get_mut(active) {
            return child.handle(event);
        }
        HandleResult::Ignored
    }
}

#[cfg(test)]
mod tests;
