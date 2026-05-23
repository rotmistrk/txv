//! Glyph set system — semantic character roles with ASCII/Unicode/Nerd tiers.
//!
//! Widgets call `glyphs()` to get the active glyph set. Set once at startup,
//! swappable at runtime via `set_glyphs()`.
//!
//! ## Tiers
//!
//! - **ASCII** — works on any terminal, no Unicode required
//! - **Unicode** — standard box-drawing and symbols (safe subset)
//! - **UnicodeExtended** — richer Unicode (rounded corners, etc.)
//! - **Nerd** — Nerd Font icons (requires Nerd Font installed)
//!
//! ## Usage
//!
//! ```rust
//! use txv_core::glyphs::{glyphs, GlyphSet};
//!
//! let g = glyphs();
//! // Use semantic roles:
//! let corner = g.box_drawing.tl; // '┌' in Unicode, '+' in ASCII
//! ```

mod defaults;
mod defs;

#[cfg(test)]
mod tests;

use std::sync::OnceLock;

pub use defs::{BoxGlyphs, ChromeGlyphs, GlyphSet, GlyphTier, ProgressGlyphs, TreeGlyphs, UiGlyphs};

static GLYPHS: OnceLock<std::sync::RwLock<GlyphSet>> = OnceLock::new();

/// Get the active glyph set.
pub fn glyphs() -> GlyphSet {
    match GLYPHS.get() {
        Some(lock) => lock.read().map(|g| g.clone()).unwrap_or_default(),
        None => GlyphSet::default(),
    }
}

/// Set the active glyph set.
pub fn set_glyphs(g: GlyphSet) {
    match GLYPHS.get() {
        Some(lock) => {
            if let Ok(mut w) = lock.write() {
                *w = g;
            }
        }
        None => {
            let _ = GLYPHS.set(std::sync::RwLock::new(g));
        }
    }
}

/// Detect best glyph tier for the current terminal.
/// Checks TERM_PROGRAM and font hints; falls back to Unicode.
pub fn detect_glyph_tier() -> GlyphTier {
    // Check for Nerd Font hints
    if std::env::var("NERD_FONT").is_ok() {
        return GlyphTier::Nerd;
    }
    // Check terminal — most modern terminals support Unicode
    if let Ok(term) = std::env::var("TERM") {
        if term == "dumb" || term == "vt100" {
            return GlyphTier::Ascii;
        }
    }
    GlyphTier::Unicode
}
