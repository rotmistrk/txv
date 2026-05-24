//! Key binding export for TiledWorkspace.

use super::commands::*;
use super::{KeyBinding, TiledWorkspace};

impl TiledWorkspace {
    /// Export default key→command bindings for registration with a status bar.
    /// All entries have no payload — commands are self-describing.
    pub fn default_bindings(&self) -> Vec<KeyBinding> {
        let km = &self.keymap;
        vec![
            (km.toggle_tree, CM_TW_TOGGLE_TREE, None),
            (km.toggle_tools, CM_TW_TOGGLE_TOOLS, None),
            (km.zoom, CM_TW_ZOOM, None),
            (km.layout_cycle, CM_TW_LAYOUT_CYCLE, None),
            (km.focus_left, CM_TW_FOCUS_LEFT, None),
            (km.focus_right, CM_TW_FOCUS_RIGHT, None),
            (km.focus_up, CM_TW_FOCUS_UP, None),
            (km.focus_down, CM_TW_FOCUS_DOWN, None),
            (km.resize_left, CM_TW_SHRINK_H, None),
            (km.resize_right, CM_TW_GROW_H, None),
            (km.resize_up, CM_TW_SHRINK_V, None),
            (km.resize_down, CM_TW_GROW_V, None),
            (km.tab_dropdown, CM_TW_TAB_DROPDOWN, None),
            (km.tab_dropdown_up, CM_TW_TAB_DROPDOWN_UP, None),
            (km.tab_dropdown_down, CM_TW_TAB_DROPDOWN_DOWN, None),
            (km.tab_next, CM_TW_TAB_NEXT, None),
            (km.tab_prev, CM_TW_TAB_PREV, None),
            (km.tab_close, CM_TW_TAB_CLOSE, None),
            (km.subpanel_focus, CM_TW_CYCLE_SUBPANEL, None),
            (km.subpanel_move_tab, CM_TW_MOVE_TAB_SUBPANEL, None),
            (km.subpanel_grow, CM_TW_GROW_SUBPANEL, None),
            (km.subpanel_shrink, CM_TW_SHRINK_SUBPANEL, None),
        ]
    }
}
