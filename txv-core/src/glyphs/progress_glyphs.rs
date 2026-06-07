//! Progress bar characters.

/// Progress bar characters.
#[derive(Clone, Debug)]
pub struct ProgressGlyphs {
    pub(crate) filled: char,
    pub(crate) empty: char,
    pub(crate) partial: char,
}

impl ProgressGlyphs {
    pub fn filled(&self) -> char {
        self.filled
    }
    pub fn empty(&self) -> char {
        self.empty
    }
    pub fn partial(&self) -> char {
        self.partial
    }
}
