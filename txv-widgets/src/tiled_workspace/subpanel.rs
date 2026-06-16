//! Subpanel management and state persistence.

use super::types::SplitNode;
use super::workspace_state::WorkspaceState;
use super::TiledWorkspace;
use crate::split_panel::SplitPanel;
use crate::tab_panel::TabPanel;

impl TiledWorkspace {
    /// Split the focused panel in place: places `view` in a new second
    /// TabPanel subpanel. The original TabPanel stays in place.
    /// Returns true if split was created.
    pub fn split_in_place(&mut self, view: Box<dyn txv_core::prelude::View>, title: &str) -> bool {
        let idx = self.group.focused_index();
        if idx >= self.configs.len() || !self.configs[idx].splittable {
            return false;
        }
        let mode = self.configs[idx].tab_mode;
        let Some(sp) = self.split_panel_mut(idx) else {
            return false;
        };
        // Only split if currently unsplit (1 child = single TabPanel)
        if sp.child_count() != 1 {
            return false;
        }
        let mut new_tp = TabPanel::new(mode);
        new_tp.insert_tab(title, view);
        sp.add_child(Box::new(new_tp), 0.5);
        sp.equalize();
        sp.set_focused(0); // keep focus on original pane
        self.recompute_layout();
        true
    }

    /// Collapse the focused panel's subpanel split: removes the focused
    /// subpanel's TabPanel and promotes the remaining one as the sole child.
    /// Returns the removed TabPanel's active view (if any).
    pub fn collapse_subpanel(&mut self) -> Option<Box<dyn txv_core::prelude::View>> {
        let idx = self.group.focused_index();
        if idx >= self.configs.len() || !self.configs[idx].splittable {
            return None;
        }
        let sp = self.split_panel_mut(idx)?;
        if sp.child_count() < 2 {
            return None;
        }
        let focused = sp.focused_index();
        // Remove the focused subpanel
        let mut removed = sp.remove_child(focused)?;
        // Remove any postprocess children (ScrollSyncView etc.)
        while sp.child_count() > 1 {
            sp.remove_child(sp.child_count() - 1);
        }
        self.recompute_layout();
        // Extract the active view from the removed TabPanel
        removed
            .as_any_mut()
            .and_then(|a| a.downcast_mut::<TabPanel>())
            .and_then(|tp| tp.close_active())
    }

    /// Close the OTHER subpanel (keep focused). Like vim's :only.
    pub fn collapse_other_subpanel(&mut self) -> Option<Box<dyn txv_core::prelude::View>> {
        let idx = self.group.focused_index();
        if idx >= self.configs.len() || !self.configs[idx].splittable {
            return None;
        }
        let sp = self.split_panel_mut(idx)?;
        if sp.child_count() < 2 {
            return None;
        }
        let other = 1 - sp.focused_index();
        let mut removed = sp.remove_child(other)?;
        while sp.child_count() > 1 {
            sp.remove_child(sp.child_count() - 1);
        }
        self.recompute_layout();
        removed
            .as_any_mut()
            .and_then(|a| a.downcast_mut::<TabPanel>())
            .and_then(|tp| tp.close_active())
    }

    pub fn move_tab_to_subpanel(&mut self) {
        let idx = self.group.focused_index();
        if idx >= self.configs.len() || !self.configs[idx].splittable {
            return;
        }
        let Some(child) = self.group.child_mut(idx) else {
            return;
        };
        let Some(sp) = child.as_any_mut().and_then(|a| a.downcast_mut::<SplitPanel>()) else {
            return;
        };
        let mode = self.configs[idx].tab_mode;
        Self::do_move_tab(sp, mode);
        self.recompute_layout();
    }

    /// Move the active tab from the focused panel to an adjacent panel.
    /// `forward`: true = right/down, false = left/up.
    pub fn move_tab_to_adjacent(&mut self, forward: bool) {
        let current = self.group.focused_index();
        let visible: Vec<usize> = (0..self.configs.len()).filter(|&i| !self.hidden[i]).collect();
        if visible.len() <= 1 {
            return;
        }
        let pos = visible.iter().position(|&i| i == current).unwrap_or(0);
        let target = if forward {
            visible[(pos + 1) % visible.len()]
        } else {
            visible[(pos + visible.len() - 1) % visible.len()]
        };
        // Take active tab from source panel
        let tab_data = self.panel_mut(current).and_then(|p| p.take_active());
        let Some((title, view)) = tab_data else {
            return;
        };
        // Insert into target panel
        if let Some(tp) = self.panel_mut(target) {
            tp.insert_tab(title, view);
        }
        // Focus the target panel
        self.group.switch_focus(target);
        self.recompute_layout();
    }

    fn do_move_tab(sp: &mut SplitPanel, mode: crate::tab_bar::TabBarMode) {
        let focused = sp.focused_index();
        let tab_data = {
            let Some(tp) = sp.focused_child_as_mut::<TabPanel>() else {
                return;
            };
            if tp.tab_count() <= 1 {
                return;
            }
            tp.take_active()
        };
        let Some((title, view)) = tab_data else {
            return;
        };
        if sp.child_count() == 1 {
            sp.add_child(Box::new(TabPanel::new(mode)), 0.5);
            sp.equalize();
        }
        let other = if focused == 0 {
            1
        } else {
            0
        };
        if let Some(other_child) = sp.child_mut(other) {
            if let Some(tp) = other_child.as_any_mut().and_then(|a| a.downcast_mut::<TabPanel>()) {
                tp.insert_tab(title, view);
            }
        }
        sp.set_focused(other);
    }

    /// Export state for persistence.
    pub fn save_state(&self) -> WorkspaceState {
        WorkspaceState {
            wide_proportions: Self::collect_proportions(&self.wide_layout),
            narrow_proportions: Self::collect_proportions(&self.narrow_layout),
            hidden: self
                .hidden
                .iter()
                .enumerate()
                .filter(|(_, &h)| h)
                .map(|(i, _)| i)
                .collect(),
        }
    }

    /// Restore state from persistence.
    pub fn restore_state(&mut self, state: &WorkspaceState) {
        Self::apply_proportions(&mut self.wide_layout, &state.wide_proportions);
        Self::apply_proportions(&mut self.narrow_layout, &state.narrow_proportions);
        for h in &mut self.hidden {
            *h = false;
        }
        for i in 0..self.hidden.len() {
            self.group.set_child_visible(i, true);
        }
        for &id in &state.hidden {
            if id < self.hidden.len() {
                self.hidden[id] = true;
                self.group.set_child_visible(id, false);
            }
        }
        self.recompute_layout();
    }

    fn collect_proportions(node: &SplitNode) -> Vec<f32> {
        let mut out = Vec::new();
        Self::collect_proportions_inner(node, &mut out);
        out
    }

    fn collect_proportions_inner(node: &SplitNode, out: &mut Vec<f32>) {
        if let SplitNode::Split { children, .. } = node {
            for (p, child) in children {
                out.push(*p);
                Self::collect_proportions_inner(child, out);
            }
        }
    }

    fn apply_proportions(node: &mut SplitNode, props: &[f32]) {
        let mut idx = 0;
        Self::apply_proportions_inner(node, props, &mut idx);
    }

    fn apply_proportions_inner(node: &mut SplitNode, props: &[f32], idx: &mut usize) {
        if let SplitNode::Split { children, .. } = node {
            for (p, child) in children.iter_mut() {
                if *idx < props.len() {
                    *p = props[*idx];
                    *idx += 1;
                }
                Self::apply_proportions_inner(child, props, idx);
            }
        }
    }
}
