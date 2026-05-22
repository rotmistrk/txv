//! TiledWorkspace — IDE-style tiled panel layout with configurable keybindings.

pub mod keymap;
pub mod types;

mod layout;
mod view_impl;

use txv_core::prelude::*;

use crate::tab_group::TabGroup;

use keymap::WorkspaceKeymap;
use types::{PanelConfig, PanelId, SplitNode, WorkspaceState};

/// IDE-style tiled workspace with configurable panels and layout.
pub struct TiledWorkspace {
    pub(crate) group: GroupState,
    pub(crate) configs: Vec<PanelConfig>,
    pub(crate) wide_layout: SplitNode,
    pub(crate) narrow_layout: SplitNode,
    pub(crate) keymap: WorkspaceKeymap,
    pub(crate) hidden: Vec<bool>,
    pub(crate) zoomed: Option<PanelId>,
    pub(crate) wide_threshold: u16,
    pub(crate) is_wide: bool,
}

impl TiledWorkspace {
    /// Create a new workspace.
    ///
    /// `configs` defines each panel. `wide_layout` and `narrow_layout` are
    /// split trees referencing panel indices. A TabGroup is created per panel.
    pub fn new(
        configs: Vec<PanelConfig>,
        wide_layout: SplitNode,
        narrow_layout: SplitNode,
        wide_threshold: u16,
    ) -> Self {
        let panel_count = configs.len();
        let mut group = GroupState::new(ViewOptions {
            focusable: true,
            ..ViewOptions::default()
        });
        for _ in 0..panel_count {
            group.insert(Box::new(TabGroup::new()));
        }
        let hidden = vec![false; panel_count];
        Self {
            group,
            configs,
            wide_layout,
            narrow_layout,
            keymap: WorkspaceKeymap::default(),
            hidden,
            zoomed: None,
            wide_threshold,
            is_wide: true,
        }
    }

    /// Set a custom keymap.
    pub fn set_keymap(&mut self, keymap: WorkspaceKeymap) {
        self.keymap = keymap;
    }

    /// Access a panel's TabGroup.
    pub fn panel(&self, id: PanelId) -> Option<&TabGroup> {
        let child = self.group.child(id)?;
        // SAFETY: we only insert TabGroup instances
        Some(unsafe { &*(child as *const dyn View as *const TabGroup) })
    }

    /// Access a panel's TabGroup mutably.
    pub fn panel_mut(&mut self, id: PanelId) -> Option<&mut TabGroup> {
        let child = self.group.child_mut(id)?;
        let ptr: *mut dyn View = &mut **child;
        Some(unsafe { &mut *(ptr as *mut TabGroup) })
    }

    /// Insert a tab into a panel.
    pub fn insert_tab(&mut self, panel: PanelId, title: impl Into<String>, view: Box<dyn View>) {
        if let Some(p) = self.panel_mut(panel) {
            p.insert_tab(title, view);
        }
        self.recompute_layout();
    }

    /// Toggle visibility of a panel (if hideable).
    pub fn toggle_panel(&mut self, id: PanelId) {
        if id >= self.configs.len() || !self.configs[id].hideable {
            return;
        }
        self.hidden[id] = !self.hidden[id];
        // If hiding the focused panel, move focus
        if self.hidden[id] && self.group.focused_index() == id {
            self.focus_next_visible();
        }
        self.recompute_layout();
    }

    /// Toggle zoom on the focused panel.
    pub fn toggle_zoom(&mut self) {
        self.zoomed = if self.zoomed.is_some() {
            None
        } else {
            Some(self.group.focused_index())
        };
        self.recompute_layout();
    }

    /// Focus the next visible panel.
    pub fn focus_next_visible(&mut self) {
        let count = self.configs.len();
        let start = self.group.focused_index();
        for offset in 1..count {
            let idx = (start + offset) % count;
            if !self.hidden[idx] {
                self.group.switch_focus(idx);
                return;
            }
        }
    }

    /// Focus a specific panel by ID.
    pub fn focus_panel(&mut self, id: PanelId) {
        if id < self.configs.len() && !self.hidden[id] {
            self.group.switch_focus(id);
        }
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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
