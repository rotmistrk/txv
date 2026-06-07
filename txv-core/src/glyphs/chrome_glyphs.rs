//! Chrome/tab bar characters.

/// Chrome/tab bar characters.
#[derive(Clone, Debug)]
pub struct ChromeGlyphs {
    pub(crate) tab_left: &'static str,
    pub(crate) tab_right: &'static str,
    pub(crate) tab_separator: &'static str,
    pub(crate) tab_separator_left: &'static str,
    pub(crate) dropdown_arrow: &'static str,
    pub(crate) badge_busy: &'static str,
    pub(crate) badge_idle: &'static str,
    pub(crate) badge_exited: &'static str,
}

impl ChromeGlyphs {
    pub fn tab_left(&self) -> &'static str {
        self.tab_left
    }
    pub fn tab_right(&self) -> &'static str {
        self.tab_right
    }
    pub fn tab_separator(&self) -> &'static str {
        self.tab_separator
    }
    pub fn tab_separator_left(&self) -> &'static str {
        self.tab_separator_left
    }
    pub fn dropdown_arrow(&self) -> &'static str {
        self.dropdown_arrow
    }
    pub fn badge_busy(&self) -> &'static str {
        self.badge_busy
    }
    pub fn badge_idle(&self) -> &'static str {
        self.badge_idle
    }
    pub fn badge_exited(&self) -> &'static str {
        self.badge_exited
    }
}
