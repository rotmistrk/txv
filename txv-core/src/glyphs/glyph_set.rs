//! GlyphSet — complete glyph set.

use super::{BoxGlyphs, ChromeGlyphs, ProgressGlyphs, TreeGlyphs, UiGlyphs};

/// Which tier of glyphs to use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GlyphTier {
    Ascii,
    Unicode,
    UnicodeExtended,
    Nerd,
}

/// Complete glyph set — all semantic character roles.
#[derive(Clone, Debug)]
pub struct GlyphSet {
    pub(crate) tier: GlyphTier,
    pub(crate) box_drawing: BoxGlyphs,
    pub(crate) tree: TreeGlyphs,
    pub(crate) ui: UiGlyphs,
    pub(crate) chrome: ChromeGlyphs,
    pub(crate) progress: ProgressGlyphs,
}

impl GlyphSet {
    pub fn tier(&self) -> GlyphTier {
        self.tier
    }

    pub fn box_drawing(&self) -> &BoxGlyphs {
        &self.box_drawing
    }

    pub fn tree(&self) -> &TreeGlyphs {
        &self.tree
    }

    pub fn ui(&self) -> &UiGlyphs {
        &self.ui
    }

    pub fn chrome(&self) -> &ChromeGlyphs {
        &self.chrome
    }

    pub fn progress(&self) -> &ProgressGlyphs {
        &self.progress
    }
}
