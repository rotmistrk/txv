//! ToolsPanel — a panel that can split into multiple subpanels,
//! each with its own TabGroup. Supports split-on-move, focus cycling,
//! and proportional resize between subpanels.

use txv_core::prelude::*;

use crate::tab_group::TabGroup;
use crate::tiled_workspace::types::SplitDir;

/// A panel containing 1..N TabGroup subpanels in a split arrangement.
pub struct ToolsPanel {
    state: ViewState,
    subpanels: Vec<TabGroup>,
    proportions: Vec<f32>,
    focused: usize,
    split_dir: SplitDir,
}

impl ToolsPanel {
    pub fn new(split_dir: SplitDir) -> Self {
        Self {
            state: ViewState::new(ViewOptions {
                focusable: true,
                ..ViewOptions::default()
            }),
            subpanels: vec![TabGroup::new()],
            proportions: vec![1.0],
            focused: 0,
            split_dir,
        }
    }

    /// Set split direction (changes on layout switch).
    pub fn set_split_dir(&mut self, dir: SplitDir) {
        if self.split_dir != dir {
            self.split_dir = dir;
            self.relayout();
        }
    }

    /// Number of subpanels.
    pub fn subpanel_count(&self) -> usize {
        self.subpanels.len()
    }

    /// Access the focused subpanel's TabGroup.
    pub fn focused_subpanel(&self) -> &TabGroup {
        &self.subpanels[self.focused]
    }

    /// Access the focused subpanel's TabGroup mutably.
    pub fn focused_subpanel_mut(&mut self) -> &mut TabGroup {
        &mut self.subpanels[self.focused]
    }

    /// Insert a tab into the focused subpanel.
    pub fn insert_tab(&mut self, title: impl Into<String>, view: Box<dyn View>) {
        self.subpanels[self.focused].insert_tab(title, view);
    }

    /// Total tab count across all subpanels.
    pub fn tab_count(&self) -> usize {
        self.subpanels.iter().map(|s| s.tab_count()).sum()
    }

    /// Cycle focus to the next subpanel.
    pub fn cycle_focus(&mut self) {
        if self.subpanels.len() > 1 {
            self.subpanels[self.focused].group.unselect_focused();
            self.focused = (self.focused + 1) % self.subpanels.len();
            self.subpanels[self.focused].group.select_focused();
            self.state.mark_dirty();
        }
    }

    /// Move active tab from focused subpanel to next. Creates split if needed.
    pub fn move_tab_to_next(&mut self) {
        let src = self.focused;
        if self.subpanels[src].tab_count() == 0 {
            return;
        }

        // Split-on-move: create second subpanel if only one exists
        if self.subpanels.len() == 1 {
            self.subpanels.push(TabGroup::new());
            self.proportions = vec![0.5, 0.5];
            self.relayout();
        }

        let dst = (src + 1) % self.subpanels.len();
        let tab_idx = self.subpanels[src].active_index();
        let title = self.subpanels[src].tab_title(tab_idx).unwrap_or("").to_string();
        if let Some(view) = self.subpanels[src].take_tab(tab_idx) {
            self.subpanels[dst].insert_tab(title, view);
        }

        // Auto-unsplit: remove empty subpanels
        self.cleanup_empty();
        self.state.mark_dirty();
    }

    /// Grow the focused subpanel.
    pub fn grow_focused(&mut self) {
        self.adjust_size(0.05);
    }

    /// Shrink the focused subpanel.
    pub fn shrink_focused(&mut self) {
        self.adjust_size(-0.05);
    }

    fn adjust_size(&mut self, delta: f32) {
        if self.subpanels.len() < 2 {
            return;
        }
        let neighbor = if self.focused + 1 < self.subpanels.len() {
            self.focused + 1
        } else {
            self.focused - 1
        };
        self.proportions[self.focused] = (self.proportions[self.focused] + delta).clamp(0.1, 0.9);
        self.proportions[neighbor] = (self.proportions[neighbor] - delta).clamp(0.1, 0.9);
        // Normalize
        let total: f32 = self.proportions.iter().sum();
        for p in &mut self.proportions {
            *p /= total;
        }
        self.relayout();
    }

    fn cleanup_empty(&mut self) {
        // Remove empty subpanels (but keep at least one)
        let mut i = 0;
        while self.subpanels.len() > 1 && i < self.subpanels.len() {
            if self.subpanels[i].tab_count() == 0 {
                self.subpanels.remove(i);
                self.proportions.remove(i);
                if self.focused >= self.subpanels.len() {
                    self.focused = self.subpanels.len() - 1;
                }
            } else {
                i += 1;
            }
        }
        // Normalize proportions
        let total: f32 = self.proportions.iter().sum();
        if total > 0.0 {
            for p in &mut self.proportions {
                *p /= total;
            }
        }
    }

    fn relayout(&mut self) {
        let b = self.state.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        let mut offset = 0u16;
        let total_size = match self.split_dir {
            SplitDir::Horizontal => b.h,
            SplitDir::Vertical => b.w,
        };
        let count = self.subpanels.len();
        for (i, panel) in self.subpanels.iter_mut().enumerate() {
            let is_last = i == count - 1;
            let size = if is_last {
                total_size.saturating_sub(offset)
            } else {
                (total_size as f32 * self.proportions[i]) as u16
            };
            let rect = match self.split_dir {
                SplitDir::Horizontal => Rect::new(b.x, b.y + offset, b.w, size),
                SplitDir::Vertical => Rect::new(b.x + offset, b.y, size, b.h),
            };
            panel.set_bounds(rect);
            offset += size;
        }
        self.state.mark_dirty();
    }
}

impl View for ToolsPanel {
    delegate_view_state!(state, override { set_bounds, draw, handle });

    fn set_bounds(&mut self, r: Rect) {
        self.state.set_bounds(r);
        self.relayout();
    }

    fn draw(&mut self) {
        let b = self.state.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        self.state.buffer_mut().fill(' ', Style::default());
        for panel in &mut self.subpanels {
            panel.draw();
        }
        let buf_ptr = self.state.buffer_mut() as *mut Buffer;
        for panel in &self.subpanels {
            let cb = panel.group.bounds();
            if cb.w == 0 || cb.h == 0 {
                continue;
            }
            let dx = cb.x.saturating_sub(b.x);
            let dy = cb.y.saturating_sub(b.y);
            unsafe { (*buf_ptr).blit(panel.group.buffer(), dx, dy) };
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // Tick goes to all subpanels
        if matches!(event, Event::Tick) {
            for panel in &mut self.subpanels {
                panel.handle(event);
            }
            return HandleResult::Ignored;
        }
        // Dispatch to focused subpanel
        self.subpanels[self.focused].handle(event)
    }
}

#[cfg(test)]
#[path = "tools_panel_tests.rs"]
mod tests;
