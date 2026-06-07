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
pub mod workspace_state;

mod accessors;
mod bindings;
mod chrome;
mod handle_cmd;
mod handle_focus;
mod layout;
mod view_impl;

use std::any::Any;

use txv_core::event::CommandId;
use txv_core::prelude::*;

use crate::tab_panel::TabPanel;

use keymap::WorkspaceKeymap;
use types::{LayoutMode, PanelConfig, PanelId, SplitNode};

/// A key binding entry: (key, command_id, optional payload).
pub type KeyBinding = (KeyEvent, CommandId, Option<Box<dyn Any + Send>>);

/// IDE-style tiled workspace with configurable panels and layout.
pub struct TiledWorkspace {
    group: GroupState,
    configs: Vec<PanelConfig>,
    wide_layout: SplitNode,
    narrow_layout: SplitNode,
    keymap: WorkspaceKeymap,
    hidden: Vec<bool>,
    zoomed: Option<PanelId>,
    wide_threshold: u16,
    narrow_threshold: u16,
    layout_mode: LayoutMode,
    is_wide: bool,
    handle_keys: bool,
    h_divider_gaps: bool,
    v_divider_gaps: bool,
}

impl TiledWorkspace {
    /// Create a new workspace.
    ///
    /// `configs` defines each panel. `wide_layout` and `narrow_layout` are
    /// split trees referencing panel indices. Each panel is a SplitPanel
    /// containing one TabPanel (unsplit). Split adds a second TabPanel child.
    pub fn new(
        configs: Vec<PanelConfig>,
        wide_layout: SplitNode,
        narrow_layout: SplitNode,
        wide_threshold: u16,
    ) -> Self {
        let panel_count = configs.len();
        let mut group = GroupState::new(ViewOptions::default().with_focusable());
        for cfg in &configs {
            let dir = match cfg.position {
                types::PanelPosition::Right => types::SplitDir::Vertical,
                _ => types::SplitDir::Horizontal,
            };
            let mut sp = crate::split_panel::SplitPanel::new(dir);
            sp.set_chrome_row(true);
            sp.add_child(Box::new(TabPanel::new(cfg.tab_mode)), 1.0);
            group.insert(Box::new(sp));
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

    /// Get the current keymap.
    pub fn keymap(&self) -> &WorkspaceKeymap {
        &self.keymap
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
        // Try direct TabPanel first, then look inside SplitPanel
        if let Some(tp) = child.as_any().and_then(|a| a.downcast_ref::<TabPanel>()) {
            return Some(tp);
        }
        if let Some(sp) = child
            .as_any()
            .and_then(|a| a.downcast_ref::<crate::split_panel::SplitPanel>())
        {
            return sp.focused_child_as::<TabPanel>();
        }
        None
    }

    /// Access a panel's TabPanel mutably.
    pub fn panel_mut(&mut self, id: PanelId) -> Option<&mut TabPanel> {
        let child = self.group.child_mut(id)?;
        // Try direct TabPanel first
        if child.as_any_mut().and_then(|a| a.downcast_mut::<TabPanel>()).is_some() {
            // Re-borrow to satisfy borrow checker
            return child.as_any_mut().and_then(|a| a.downcast_mut::<TabPanel>());
        }
        // Try inside SplitPanel
        if let Some(sp) = child
            .as_any_mut()
            .and_then(|a| a.downcast_mut::<crate::split_panel::SplitPanel>())
        {
            return sp.focused_child_as_mut::<TabPanel>();
        }
        None
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

    /// Mutable access to the workspace buffer (for chrome drawing).
    pub fn buffer_mut(&mut self) -> &mut Buffer {
        self.group.buffer_mut()
    }

    /// Focus a specific panel by ID.
    pub fn focus_panel(&mut self, id: PanelId) {
        if id < self.configs.len() && !self.hidden[id] {
            self.group.switch_focus(id);
            if self.zoomed.is_some() {
                self.zoomed = Some(id);
                self.recompute_layout();
            }
        }
    }

    /// Run a closure on the focused panel's SplitPanel (if it is one and splittable).
    pub fn with_split_panel(&mut self, f: impl FnOnce(&mut crate::split_panel::SplitPanel)) {
        if let Some(sp) = self.split_panel_mut(self.group.focused_index()) {
            f(sp);
        }
    }

    /// Get mutable access to a panel's underlying SplitPanel by panel ID.
    pub fn split_panel_mut(&mut self, id: PanelId) -> Option<&mut crate::split_panel::SplitPanel> {
        if id >= self.configs.len() || !self.configs[id].splittable {
            return None;
        }
        self.group
            .child_mut(id)?
            .as_any_mut()
            .and_then(|a| a.downcast_mut::<crate::split_panel::SplitPanel>())
    }

    /// Get immutable access to a panel's underlying SplitPanel by panel ID.
    pub fn split_panel(&self, id: PanelId) -> Option<&crate::split_panel::SplitPanel> {
        if id >= self.configs.len() || !self.configs[id].splittable {
            return None;
        }
        self.group
            .child(id)?
            .as_any()
            .and_then(|a| a.downcast_ref::<crate::split_panel::SplitPanel>())
    }

    // move_tab_to_subpanel, save_state, restore_state are in subpanel.rs
}

mod subpanel;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
