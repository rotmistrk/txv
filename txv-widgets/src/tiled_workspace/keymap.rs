//! Configurable keybindings for TiledWorkspace.

use txv_core::event::{KeyCode, KeyEvent, KeyMod};

/// All configurable workspace actions.
#[derive(Clone, Debug)]
pub struct WorkspaceKeymap {
    pub(crate) toggle_tree: KeyEvent,
    pub(crate) toggle_tools: KeyEvent,
    pub(crate) zoom: KeyEvent,
    pub(crate) focus_left: KeyEvent,
    pub(crate) focus_right: KeyEvent,
    pub(crate) focus_up: KeyEvent,
    pub(crate) focus_down: KeyEvent,
    pub(crate) resize_left: KeyEvent,
    pub(crate) resize_right: KeyEvent,
    pub(crate) resize_up: KeyEvent,
    pub(crate) resize_down: KeyEvent,
    pub(crate) tab_dropdown: KeyEvent,
    pub(crate) tab_dropdown_up: KeyEvent,
    pub(crate) tab_dropdown_down: KeyEvent,
    pub(crate) tab_next: KeyEvent,
    pub(crate) tab_prev: KeyEvent,
    pub(crate) tab_close: KeyEvent,
    pub(crate) subpanel_focus: KeyEvent,
    pub(crate) subpanel_move_tab: KeyEvent,
    pub(crate) subpanel_grow: KeyEvent,
    pub(crate) subpanel_shrink: KeyEvent,
    pub(crate) layout_cycle: KeyEvent,
}

impl WorkspaceKeymap {
    pub fn matches(&self, key: &KeyEvent, action: &KeyEvent) -> bool {
        key.code() == action.code() && key.modifiers() == action.modifiers()
    }

    pub fn set_focus_up(&mut self, key: KeyEvent) {
        self.focus_up = key;
    }

    pub fn set_focus_down(&mut self, key: KeyEvent) {
        self.focus_down = key;
    }

    pub fn set_tab_dropdown_up(&mut self, key: KeyEvent) {
        self.tab_dropdown_up = key;
    }

    pub fn set_tab_dropdown_down(&mut self, key: KeyEvent) {
        self.tab_dropdown_down = key;
    }

    pub fn set_subpanel_focus(&mut self, key: KeyEvent) {
        self.subpanel_focus = key;
    }

    pub fn set_subpanel_grow(&mut self, key: KeyEvent) {
        self.subpanel_grow = key;
    }

    pub fn set_subpanel_shrink(&mut self, key: KeyEvent) {
        self.subpanel_shrink = key;
    }
}

fn alt(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyMod::ALT)
}

fn ctrl_shift(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyMod::CTRL.with_shift())
}

fn alt_shift(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyMod::ALT.with_shift())
}

fn ctrl(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyMod::CTRL)
}

fn ctrl_alt(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyMod::CTRL.with_alt())
}

impl Default for WorkspaceKeymap {
    fn default() -> Self {
        Self {
            toggle_tree: alt(KeyCode::Char(',')),
            toggle_tools: alt(KeyCode::Char('.')),
            zoom: alt(KeyCode::Char('/')),
            focus_left: ctrl_shift(KeyCode::Left),
            focus_right: ctrl_shift(KeyCode::Right),
            focus_up: ctrl_alt(KeyCode::Up),
            focus_down: ctrl_alt(KeyCode::Down),
            resize_left: alt_shift(KeyCode::Left),
            resize_right: alt_shift(KeyCode::Right),
            resize_up: alt_shift(KeyCode::Up),
            resize_down: alt_shift(KeyCode::Down),
            tab_dropdown: alt(KeyCode::Char('0')),
            tab_dropdown_up: ctrl_shift(KeyCode::Up),
            tab_dropdown_down: ctrl_shift(KeyCode::Down),
            tab_next: alt(KeyCode::Char(';')),
            tab_prev: alt(KeyCode::Char('\'')),
            tab_close: alt(KeyCode::Char('w')),
            subpanel_focus: ctrl(KeyCode::Char('w')),
            subpanel_move_tab: ctrl_alt(KeyCode::Char('w')),
            subpanel_grow: alt(KeyCode::Char('=')),
            subpanel_shrink: alt(KeyCode::Char('-')),
            layout_cycle: alt(KeyCode::Char('\\')),
        }
    }
}
