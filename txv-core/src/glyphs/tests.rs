//! Glyph set tests.

use super::*;

#[test]
fn default_is_unicode() {
    let g = GlyphSet::default();
    assert_eq!(g.tier, GlyphTier::Unicode);
    assert_eq!(g.box_drawing.tl, '┌');
    assert_eq!(g.box_drawing.h_heavy, '═');
}

#[test]
fn ascii_uses_only_ascii_chars() {
    let g = GlyphSet::ascii();
    assert!(g.box_drawing.h.is_ascii());
    assert!(g.box_drawing.v.is_ascii());
    assert!(g.box_drawing.tl.is_ascii());
    assert!(g.tree.expanded.is_ascii());
    assert!(g.tree.collapsed.is_ascii());
    assert!(g.ui.scrollbar_track.is_ascii());
    assert!(g.progress.filled.is_ascii());
}

#[test]
fn unicode_extended_has_rounded_corners() {
    let g = GlyphSet::unicode_extended();
    assert_eq!(g.box_drawing.tl_round, '╭');
    assert_eq!(g.box_drawing.br_round, '╯');
    // Light corners remain standard
    assert_eq!(g.box_drawing.tl, '┌');
}

#[test]
fn nerd_has_special_tree_icons() {
    let g = GlyphSet::nerd();
    assert_eq!(g.tier, GlyphTier::Nerd);
    // Nerd tree glyphs are multi-byte
    assert!(!g.tree.expanded.is_ascii());
}

#[test]
fn from_tier_round_trips() {
    for tier in [
        GlyphTier::Ascii,
        GlyphTier::Unicode,
        GlyphTier::UnicodeExtended,
        GlyphTier::Nerd,
    ] {
        let g = GlyphSet::from_tier(tier);
        assert_eq!(g.tier, tier);
    }
}

#[test]
fn set_and_get_glyphs() {
    set_glyphs(GlyphSet::ascii());
    let g = glyphs();
    assert_eq!(g.tier, GlyphTier::Ascii);
    // Restore
    set_glyphs(GlyphSet::default());
}
