//! Subpanel management and state persistence.

use super::types::SplitNode;
use super::{TiledWorkspace, WorkspaceState};
use crate::tab_panel::TabPanel;

impl TiledWorkspace {
    pub fn move_tab_to_subpanel(&mut self) {
        let idx = self.group.focused_index();
        if idx >= self.configs.len() || !self.configs[idx].splittable {
            return;
        }
        let Some(child) = self.group.child_mut(idx) else {
            return;
        };
        let Some(sp) = child
            .as_any_mut()
            .and_then(|a| a.downcast_mut::<crate::split_panel::SplitPanel>())
        else {
            return;
        };

        // Take the active tab from the focused TabPanel
        let focused = sp.focused_index();
        let tab_data = {
            let Some(tp) = sp.focused_child_as_mut::<TabPanel>() else {
                return;
            };
            if tp.tab_count() <= 1 {
                return; // don't leave a panel empty
            }
            tp.take_active()
        };
        let Some((title, view)) = tab_data else {
            return;
        };

        // If only one child, add a second TabPanel
        if sp.child_count() == 1 {
            let mode = self.configs[idx].tab_mode;
            sp.add_child(Box::new(TabPanel::new(mode)), 0.5);
            sp.set_proportion(0, 0.5);
        }

        // Insert into the other TabPanel
        let other = if focused == 0 {
            1
        } else {
            0
        };
        if let Some(other_child) = sp.child_mut(other) {
            if let Some(other_tp) = other_child.as_any_mut().and_then(|a| a.downcast_mut::<TabPanel>()) {
                other_tp.insert_tab(title, view);
            }
        }
        sp.set_focused(other);
        self.recompute_layout();
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
        for &id in &state.hidden {
            if id < self.hidden.len() {
                self.hidden[id] = true;
            }
        }
        self.recompute_layout();
    }

    fn collect_proportions(node: &SplitNode) -> Vec<f32> {
        match node {
            SplitNode::Leaf(_) => vec![],
            SplitNode::Split { children, .. } => {
                let mut out: Vec<f32> = children.iter().map(|(p, _)| *p).collect();
                for (_, child) in children {
                    out.extend(Self::collect_proportions(child));
                }
                out
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
