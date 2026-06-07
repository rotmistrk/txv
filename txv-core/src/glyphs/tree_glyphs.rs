//! Tree view characters.

/// Tree view characters.
#[derive(Clone, Debug)]
pub struct TreeGlyphs {
    pub(crate) expanded: &'static str,
    pub(crate) collapsed: &'static str,
    pub(crate) branch: char,
    pub(crate) last_branch: char,
    pub(crate) pipe: char,
    pub(crate) horizontal: char,
    pub(crate) open_indicator: &'static str,
}

impl TreeGlyphs {
    pub fn expanded(&self) -> &'static str {
        self.expanded
    }
    pub fn collapsed(&self) -> &'static str {
        self.collapsed
    }
    pub fn branch(&self) -> char {
        self.branch
    }
    pub fn last_branch(&self) -> char {
        self.last_branch
    }
    pub fn pipe(&self) -> char {
        self.pipe
    }
    pub fn horizontal(&self) -> char {
        self.horizontal
    }
    pub fn open_indicator(&self) -> &'static str {
        self.open_indicator
    }
}
