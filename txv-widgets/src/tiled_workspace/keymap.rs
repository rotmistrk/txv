//! Configurable keybindings for TiledWorkspace.

use txv_core::event::{KeyCode, KeyEvent, KeyMod};

/// All configurable workspace actions.
#[derive(Clone, Debug)]
pub struct WorkspaceKeymap {
    pub toggle_tree: KeyEvent,
    pub toggle_tools: KeyEvent,
    pub zoom: KeyEvent,
    pub focus_left: KeyEvent,
    pub focus_right: KeyEvent,
    pub focus_up: KeyEvent,
    pub focus_down: KeyEvent,
    pub resize_left: KeyEvent,
    pub resize_right: KeyEvent,
    pub resize_up: KeyEvent,
    pub resize_down: KeyEvent,
    pub tab_dropdown: KeyEvent,
    pub subpanel_focus: KeyEvent,
    pub subpanel_move_tab: KeyEvent,
    pub subpanel_grow: KeyEvent,
    pub subpanel_shrink: KeyEvent,
}

impl WorkspaceKeymap {
    pub fn matches(&self, key: &KeyEvent, action: &KeyEvent) -> bool {
        key.code == action.code && key.modifiers == action.modifiers
    }
}

fn alt(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyMod {
            alt: true,
            ctrl: false,
            shift: false,
        },
    }
}

fn ctrl_shift(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyMod {
            ctrl: true,
            shift: true,
            alt: false,
        },
    }
}

fn alt_shift(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyMod {
            alt: true,
            shift: true,
            ctrl: false,
        },
    }
}

fn ctrl(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyMod {
            ctrl: true,
            shift: false,
            alt: false,
        },
    }
}

fn ctrl_alt(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyMod {
            ctrl: true,
            alt: true,
            shift: false,
        },
    }
}

impl Default for WorkspaceKeymap {
    fn default() -> Self {
        Self {
            toggle_tree: alt(KeyCode::Char(',')),
            toggle_tools: alt(KeyCode::Char('.')),
            zoom: alt(KeyCode::Char('/')),
            focus_left: ctrl_shift(KeyCode::Left),
            focus_right: ctrl_shift(KeyCode::Right),
            focus_up: ctrl_shift(KeyCode::Up),
            focus_down: ctrl_shift(KeyCode::Down),
            resize_left: alt_shift(KeyCode::Left),
            resize_right: alt_shift(KeyCode::Right),
            resize_up: alt_shift(KeyCode::Up),
            resize_down: alt_shift(KeyCode::Down),
            tab_dropdown: alt(KeyCode::Char('0')),
            subpanel_focus: ctrl(KeyCode::Char('w')),
            subpanel_move_tab: ctrl_alt(KeyCode::Char('w')),
            subpanel_grow: alt(KeyCode::Char('=')),
            subpanel_shrink: alt(KeyCode::Char('-')),
        }
    }
}
