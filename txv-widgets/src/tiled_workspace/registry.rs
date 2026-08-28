//! Command registration for TiledWorkspace.

use txv_core::command_registry::{register, CommandMeta};

use super::commands::{
    CM_TW_CYCLE_SUBPANEL, CM_TW_FOCUS_DOWN, CM_TW_FOCUS_LEFT, CM_TW_FOCUS_RIGHT, CM_TW_FOCUS_UP, CM_TW_GROW_H,
    CM_TW_GROW_SUBPANEL, CM_TW_GROW_V, CM_TW_LAYOUT_CYCLE, CM_TW_MOVE_TAB_SUBPANEL, CM_TW_SHRINK_H,
    CM_TW_SHRINK_SUBPANEL, CM_TW_SHRINK_V, CM_TW_TAB_CLOSE, CM_TW_TAB_DROPDOWN, CM_TW_TAB_DROPDOWN_CLOSE,
    CM_TW_TAB_DROPDOWN_DOWN, CM_TW_TAB_DROPDOWN_UP, CM_TW_TAB_NEXT, CM_TW_TAB_PREV, CM_TW_TOGGLE_TOOLS,
    CM_TW_TOGGLE_TREE, CM_TW_ZOOM,
};

/// Register all TiledWorkspace commands with the command registry.
/// Call this once at application startup.
pub fn register_commands() {
    register_panel_visibility();
    register_focus_navigation();
    register_resize();
    register_tabs();
    register_layout();
    register_subpanels();
}

fn register_panel_visibility() {
    register(
        CM_TW_TOGGLE_TREE,
        CommandMeta::new("toggle-tree", "Toggle tree panel", "Show or hide the tree panel"),
    );
    register(
        CM_TW_TOGGLE_TOOLS,
        CommandMeta::new("toggle-tools", "Toggle tools panel", "Show or hide the tools panel"),
    );
    register(
        CM_TW_ZOOM,
        CommandMeta::new("zoom", "Zoom panel", "Maximize the focused panel"),
    );
}

fn register_focus_navigation() {
    register(
        CM_TW_FOCUS_LEFT,
        CommandMeta::new("focus-left", "Focus left", "Move focus to the panel on the left"),
    );
    register(
        CM_TW_FOCUS_RIGHT,
        CommandMeta::new("focus-right", "Focus right", "Move focus to the panel on the right"),
    );
    register(
        CM_TW_FOCUS_UP,
        CommandMeta::new("focus-up", "Focus up", "Move focus to the panel above"),
    );
    register(
        CM_TW_FOCUS_DOWN,
        CommandMeta::new("focus-down", "Focus down", "Move focus to the panel below"),
    );
}

fn register_resize() {
    register(
        CM_TW_GROW_H,
        CommandMeta::new("grow-h", "Grow horizontal", "Increase the width of the focused panel"),
    );
    register(
        CM_TW_SHRINK_H,
        CommandMeta::new(
            "shrink-h",
            "Shrink horizontal",
            "Decrease the width of the focused panel",
        ),
    );
    register(
        CM_TW_GROW_V,
        CommandMeta::new("grow-v", "Grow vertical", "Increase the height of the focused panel"),
    );
    register(
        CM_TW_SHRINK_V,
        CommandMeta::new(
            "shrink-v",
            "Shrink vertical",
            "Decrease the height of the focused panel",
        ),
    );
}

fn register_tabs() {
    register(
        CM_TW_TAB_DROPDOWN,
        CommandMeta::new("tab-dropdown", "Tab dropdown", "Open the tab dropdown menu"),
    );
    register(
        CM_TW_TAB_DROPDOWN_UP,
        CommandMeta::new("tab-dropdown-up", "Dropdown up", "Move selection up in dropdown"),
    );
    register(
        CM_TW_TAB_DROPDOWN_DOWN,
        CommandMeta::new("tab-dropdown-down", "Dropdown down", "Move selection down in dropdown"),
    );
    register(
        CM_TW_TAB_DROPDOWN_CLOSE,
        CommandMeta::new("tab-dropdown-close", "Close dropdown", "Close the tab dropdown"),
    );
    register(
        CM_TW_TAB_NEXT,
        CommandMeta::new("tab-next", "Next tab", "Switch to the next tab"),
    );
    register(
        CM_TW_TAB_PREV,
        CommandMeta::new("tab-prev", "Previous tab", "Switch to the previous tab"),
    );
    register(
        CM_TW_TAB_CLOSE,
        CommandMeta::new("tab-close", "Close tab", "Close the active tab"),
    );
}

fn register_layout() {
    register(
        CM_TW_LAYOUT_CYCLE,
        CommandMeta::new("layout-cycle", "Cycle layout", "Cycle through layout modes"),
    );
}

fn register_subpanels() {
    register(
        CM_TW_CYCLE_SUBPANEL,
        CommandMeta::new("cycle-subpanel", "Cycle subpanel", "Cycle focus between subpanels"),
    );
    register(
        CM_TW_MOVE_TAB_SUBPANEL,
        CommandMeta::new("move-tab", "Move tab", "Move the active tab to the other subpanel"),
    );
    register(
        CM_TW_GROW_SUBPANEL,
        CommandMeta::new(
            "grow-subpanel",
            "Grow subpanel",
            "Increase the size of the focused subpanel",
        ),
    );
    register(
        CM_TW_SHRINK_SUBPANEL,
        CommandMeta::new(
            "shrink-subpanel",
            "Shrink subpanel",
            "Decrease the size of the focused subpanel",
        ),
    );
}
