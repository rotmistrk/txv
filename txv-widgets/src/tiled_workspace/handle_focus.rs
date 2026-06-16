//! Focus direction helpers for TiledWorkspace.

use super::TiledWorkspace;

impl TiledWorkspace {
    pub(super) fn focus_direction_zoomed(&mut self, dx: i16, dy: i16) {
        if dx > 0 || dy > 0 {
            self.focus_next_visible();
        } else {
            self.focus_prev_visible();
        }
        self.zoomed = Some(self.group.focused_index());
        self.sync_visibility();
        self.recompute_layout();
    }

    pub(super) fn focus_direction_normal(&mut self, dx: i16, dy: i16) {
        let current = self.group.focused_index();
        let forward = dx > 0 || dy > 0;

        // If subpanel focus is enabled, try cycling within the current SplitPanel first
        if self.focus_subpanels && self.try_focus_subpanel(current, forward) {
            return;
        }

        let visible: Vec<usize> = (0..self.configs.len())
            .filter(|&i| !self.hidden[i])
            .filter(|&i| {
                self.group
                    .child(i)
                    .map(|c| c.bounds().w() > 0 && c.bounds().h() > 0)
                    .unwrap_or(false)
            })
            .collect();
        if visible.len() <= 1 {
            return;
        }
        let pos = visible.iter().position(|&i| i == current).unwrap_or(0);
        let next = if forward {
            visible[(pos + 1) % visible.len()]
        } else {
            visible[(pos + visible.len() - 1) % visible.len()]
        };
        self.group.switch_focus(next);

        // When entering a new panel with subpanels, focus the first/last subpanel
        if self.focus_subpanels {
            self.enter_subpanel(next, forward);
        }
    }

    /// Try to move focus to the next/prev subpanel within the current SplitPanel.
    /// Returns true if focus moved within the split (no need to cross panels).
    fn try_focus_subpanel(&mut self, panel_id: usize, forward: bool) -> bool {
        let sp = self.split_panel_mut(panel_id);
        let Some(sp) = sp else {
            return false;
        };
        let count = sp.child_count();
        if count <= 1 {
            return false;
        }
        let focused = sp.focused_index();
        let next = if forward {
            if focused + 1 < count {
                focused + 1
            } else {
                return false;
            }
        } else {
            if focused > 0 {
                focused - 1
            } else {
                return false;
            }
        };
        sp.switch_focus(next);
        true
    }

    /// When entering a panel, focus the edge subpanel (first if forward, last if backward).
    fn enter_subpanel(&mut self, panel_id: usize, forward: bool) {
        let sp = self.split_panel_mut(panel_id);
        let Some(sp) = sp else {
            return;
        };
        let count = sp.child_count();
        if count <= 1 {
            return;
        }
        let target = if forward {
            0
        } else {
            count - 1
        };
        sp.switch_focus(target);
    }
}
