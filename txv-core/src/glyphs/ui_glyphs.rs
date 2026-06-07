//! General UI symbols.

/// General UI symbols.
#[derive(Clone, Debug)]
pub struct UiGlyphs {
    pub(crate) scrollbar_track: char,
    pub(crate) scrollbar_thumb: char,
    pub(crate) separator_h: char,
    pub(crate) separator_v: char,
    pub(crate) ellipsis: &'static str,
    pub(crate) arrow_right: &'static str,
    pub(crate) arrow_down: &'static str,
}

impl UiGlyphs {
    pub fn scrollbar_track(&self) -> char {
        self.scrollbar_track
    }
    pub fn scrollbar_thumb(&self) -> char {
        self.scrollbar_thumb
    }
    pub fn separator_h(&self) -> char {
        self.separator_h
    }
    pub fn separator_v(&self) -> char {
        self.separator_v
    }
    pub fn ellipsis(&self) -> &'static str {
        self.ellipsis
    }
    pub fn arrow_right(&self) -> &'static str {
        self.arrow_right
    }
    pub fn arrow_down(&self) -> &'static str {
        self.arrow_down
    }
}
