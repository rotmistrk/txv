//! Editor display options controlled by `:set`.

use crate::settings::CursorStyle;

/// Editor display options controlled by :set.
#[derive(Debug, Clone)]
pub struct EditorOptions {
    pub(crate) list: bool,
    pub(crate) number: bool,
    pub(crate) wrap: bool,
    pub(crate) tab_width: usize,
    pub(crate) scrolloff: usize,
    pub(crate) incsearch: bool,
    pub(crate) matchparen: bool,
    pub(crate) rainbow: bool,
    pub(crate) guides: bool,
    pub(crate) gutter_signs: bool,
    pub(crate) cursor_insert: CursorStyle,
    pub(crate) cursor_normal: CursorStyle,
    pub(crate) cursor_command: CursorStyle,
}

impl EditorOptions {
    pub fn number(&self) -> bool {
        self.number
    }
    pub fn set_number(&mut self, v: bool) {
        self.number = v;
    }
    pub fn list(&self) -> bool {
        self.list
    }
    pub fn set_list(&mut self, v: bool) {
        self.list = v;
    }
    pub fn wrap(&self) -> bool {
        self.wrap
    }
    pub fn set_wrap(&mut self, v: bool) {
        self.wrap = v;
    }
    pub fn tab_width(&self) -> usize {
        self.tab_width
    }
    pub fn set_tab_width(&mut self, v: usize) {
        self.tab_width = v;
    }
    pub fn incsearch(&self) -> bool {
        self.incsearch
    }
    pub fn set_incsearch(&mut self, v: bool) {
        self.incsearch = v;
    }
    pub fn matchparen(&self) -> bool {
        self.matchparen
    }
    pub fn set_matchparen(&mut self, v: bool) {
        self.matchparen = v;
    }
    pub fn rainbow(&self) -> bool {
        self.rainbow
    }
    pub fn set_rainbow(&mut self, v: bool) {
        self.rainbow = v;
    }
    pub fn guides(&self) -> bool {
        self.guides
    }
    pub fn set_guides(&mut self, v: bool) {
        self.guides = v;
    }
    pub fn gutter_signs(&self) -> bool {
        self.gutter_signs
    }
    pub fn set_gutter_signs(&mut self, v: bool) {
        self.gutter_signs = v;
    }
    pub fn scrolloff(&self) -> usize {
        self.scrolloff
    }
    pub fn set_scrolloff(&mut self, v: usize) {
        self.scrolloff = v;
    }
    pub fn cursor_insert(&self) -> CursorStyle {
        self.cursor_insert
    }
    pub fn set_cursor_insert(&mut self, v: CursorStyle) {
        self.cursor_insert = v;
    }
    pub fn cursor_normal(&self) -> CursorStyle {
        self.cursor_normal
    }
    pub fn set_cursor_normal(&mut self, v: CursorStyle) {
        self.cursor_normal = v;
    }
    pub fn cursor_command(&self) -> CursorStyle {
        self.cursor_command
    }
    pub fn set_cursor_command(&mut self, v: CursorStyle) {
        self.cursor_command = v;
    }
}

impl Default for EditorOptions {
    fn default() -> Self {
        Self {
            list: false,
            number: true,
            wrap: true,
            tab_width: 4,
            scrolloff: 3,
            incsearch: true,
            matchparen: true,
            rainbow: false,
            guides: false,
            gutter_signs: true,
            cursor_insert: CursorStyle::Bar,
            cursor_normal: CursorStyle::Software,
            cursor_command: CursorStyle::Software,
        }
    }
}
