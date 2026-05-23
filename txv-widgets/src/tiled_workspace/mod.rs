//! TiledWorkspace — IDE-style tiled panel layout with configurable keybindings.
//!
//! ## Command-based API
//!
//! External integrations (scripting, MCP, TCL, ex-commands) interact with
//! TiledWorkspace by emitting command events into the EventQueue — NOT by
//! calling methods directly. This keeps the architecture decoupled and
//! consistent with TXV's event-driven design.
//!
//! See [`commands`] for available command IDs and payload types.

pub mod commands;
pub mod keymap;
pub mod types;

mod bindings;
mod handle_cmd;
mod layout;
mod view_impl;

use std::any::Any;

use txv_core::event::CommandId;
use txv_core::prelude::*;

use crate::tab_panel::TabPanel;

use keymap::WorkspaceKeymap;
use types::{LayoutMode, PanelConfig, PanelId, SplitNode, WorkspaceState};

/// A key binding entry: (key, command_id, optional payload).
pub type KeyBinding = (KeyEvent, CommandId, Option<Box<dyn Any + Send>>);

/// IDE-style tiled workspace with configurable panels and layout.
pub struct TiledWorkspace {
    pub group: GroupState,
    pub(crate) configs: Vec<PanelConfig>,
    pub(crate) wide_layout: SplitNode,
    pub(crate) narrow_layout: SplitNode,
    pub(crate) keymap: WorkspaceKeymap,
    pub hidden: Vec<bool>,
    pub zoomed: Option<PanelId>,
    pub wide_threshold: u16,
    /// Below this threshold, force narrow. Between narrow and wide = hysteresis.
    pub narrow_threshold: u16,
    pub layout_mode: LayoutMode,
    pub is_wide: bool,
    pub(crate) handle_keys: bool,
    /// When false, no gap cells are reserved between split children in the
    /// horizontal direction. Default: true.
    pub h_divider_gaps: bool,
    /// When false, no gap cells are reserved between split children in the
    /// vertical direction. Default: true.
    pub v_divider_gaps: bool,
}

impl TiledWorkspace {
    /// Create a new workspace.
    ///
    /// `configs` defines each panel. `wide_layout` and `narrow_layout` are
    /// split trees referencing panel indices. A TabPanel is created per panel.
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
        for cfg in &configs {
            group.insert(Box::new(TabPanel::new(cfg.tab_mode)));
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
            narrow_threshold: wide_threshold.saturating_sub(100),
            layout_mode: LayoutMode::Auto,
            is_wide: true,
            handle_keys: true,
            h_divider_gaps: true,
            v_divider_gaps: true,
        }
    }

    /// Set a custom keymap.
    pub fn set_keymap(&mut self, keymap: WorkspaceKeymap) {
        self.keymap = keymap;
    }

    /// Disable internal key handling. When false, the workspace only
    /// responds to command events — the app/status bar owns key dispatch.
    pub fn set_handle_keys(&mut self, enabled: bool) {
        self.handle_keys = enabled;
    }

    /// Access a panel's TabPanel.
    pub fn panel(&self, id: PanelId) -> Option<&TabPanel> {
        let child = self.group.child(id)?;
        // SAFETY: we only insert TabPanel instances
        Some(unsafe { &*(child as *const dyn View as *const TabPanel) })
    }

    /// Access a panel's TabPanel mutably.
    pub fn panel_mut(&mut self, id: PanelId) -> Option<&mut TabPanel> {
        let child = self.group.child_mut(id)?;
        let ptr: *mut dyn View = &mut **child;
        Some(unsafe { &mut *(ptr as *mut TabPanel) })
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

    /// Cycle layout mode: Auto → Wide → Narrow → Auto.
    pub fn cycle_layout(&mut self) {
        self.layout_mode = match self.layout_mode {
            LayoutMode::Auto => LayoutMode::Wide,
            LayoutMode::Wide => LayoutMode::Narrow,
            LayoutMode::Narrow => LayoutMode::Auto,
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

    /// Focus the previous visible panel.
    pub fn focus_prev_visible(&mut self) {
        let count = self.configs.len();
        let start = self.group.focused_index();
        for offset in 1..count {
            let idx = (start + count - offset) % count;
            if !self.hidden[idx] {
                self.group.switch_focus(idx);
                return;
            }
        }
    }

    /// Get the currently focused panel ID.
    pub fn focused_panel(&self) -> PanelId {
        self.group.focused_index()
    }

    /// Mutable access to the workspace buffer (for chrome overlay drawing).
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.group.buffer_mut()
    }

    /// Focus a specific panel by ID.
    pub fn focus_panel(&mut self, id: PanelId) {
        if id < self.configs.len() && !self.hidden[id] {
            self.group.switch_focus(id);
        }
    }

    /// Run a closure on the focused panel's SplitPanel (if it is one).
    pub(crate) fn with_split_panel(&mut self, f: impl FnOnce(&mut crate::split_panel::SplitPanel)) {
        let idx = self.group.focused_index();
        if idx < self.configs.len() && self.configs[idx].splittable {
            if let Some(child) = self.group.child_mut(idx) {
                if let Some(sp) = child
                    .as_any_mut()
                    .and_then(|a| a.downcast_mut::<crate::split_panel::SplitPanel>())
                {
                    f(sp);
                }
            }
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
