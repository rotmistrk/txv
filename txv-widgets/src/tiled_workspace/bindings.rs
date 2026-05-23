//! Key binding export for TiledWorkspace.

use super::commands::*;
use super::types::{PanelPosition, SplitDir};
use super::{KeyBinding, TiledWorkspace};

impl TiledWorkspace {
    /// Export default key→command bindings for registration with a status bar
    /// or application-level keymap. Each entry is (key, command_id, payload).
    pub fn default_bindings(&self) -> Vec<KeyBinding> {
        let km = &self.keymap;
        let tree_id = self.find_panel_by_position(PanelPosition::Left);
        let tools_id = self
            .find_panel_by_position(PanelPosition::Right)
            .or_else(|| self.find_panel_by_position(PanelPosition::Bottom));

        let mut bindings: Vec<KeyBinding> = Vec::new();

        if let Some(id) = tree_id {
            bindings.push((km.toggle_tree, CM_TOGGLE_PANEL, Some(Box::new(id))));
        }
        if let Some(id) = tools_id {
            bindings.push((km.toggle_tools, CM_TOGGLE_PANEL, Some(Box::new(id))));
        }
        bindings.push((km.zoom, CM_ZOOM, None));
        bindings.push((km.layout_cycle, CM_LAYOUT_CYCLE, None));
        bindings.push((km.focus_left, CM_FOCUS_DIRECTION, Some(Box::new((-1i16, 0i16)))));
        bindings.push((km.focus_right, CM_FOCUS_DIRECTION, Some(Box::new((1i16, 0i16)))));
        bindings.push((km.focus_up, CM_FOCUS_DIRECTION, Some(Box::new((0i16, -1i16)))));
        bindings.push((km.focus_down, CM_FOCUS_DIRECTION, Some(Box::new((0i16, 1i16)))));
        bindings.push((
            km.resize_left,
            CM_RESIZE_PANEL,
            Some(Box::new((SplitDir::Horizontal, -1i16))),
        ));
        bindings.push((
            km.resize_right,
            CM_RESIZE_PANEL,
            Some(Box::new((SplitDir::Horizontal, 1i16))),
        ));
        bindings.push((
            km.resize_up,
            CM_RESIZE_PANEL,
            Some(Box::new((SplitDir::Vertical, -1i16))),
        ));
        bindings.push((
            km.resize_down,
            CM_RESIZE_PANEL,
            Some(Box::new((SplitDir::Vertical, 1i16))),
        ));
        bindings.push((km.tab_dropdown, CM_TAB_DROPDOWN, None));
        bindings.push((km.subpanel_focus, CM_CYCLE_SUBPANEL, None));
        bindings.push((km.subpanel_move_tab, CM_MOVE_TAB_SUBPANEL, None));
        bindings.push((km.subpanel_grow, CM_GROW_SUBPANEL, None));
        bindings.push((km.subpanel_shrink, CM_SHRINK_SUBPANEL, None));
        bindings
    }
}
