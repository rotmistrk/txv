//! Glyph set struct definitions.

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
    pub tier: GlyphTier,
    pub box_drawing: BoxGlyphs,
    pub tree: TreeGlyphs,
    pub ui: UiGlyphs,
    pub chrome: ChromeGlyphs,
    pub progress: ProgressGlyphs,
}

/// Box-drawing characters (light and heavy variants).
#[derive(Clone, Debug)]
pub struct BoxGlyphs {
    // Light
    pub h: char,
    pub v: char,
    pub tl: char,
    pub tr: char,
    pub bl: char,
    pub br: char,
    // Heavy (for dialogs/emphasis)
    pub h_heavy: char,
    pub v_heavy: char,
    pub tl_heavy: char,
    pub tr_heavy: char,
    pub bl_heavy: char,
    pub br_heavy: char,
    // Rounded (for dropdowns/popups)
    pub tl_round: char,
    pub tr_round: char,
    pub bl_round: char,
    pub br_round: char,
}

/// Tree view characters.
#[derive(Clone, Debug)]
pub struct TreeGlyphs {
    pub expanded: &'static str,
    pub collapsed: &'static str,
    pub branch: char,
    pub last_branch: char,
    pub pipe: char,
}

/// General UI symbols.
#[derive(Clone, Debug)]
pub struct UiGlyphs {
    pub scrollbar_track: char,
    pub scrollbar_thumb: char,
    pub separator_h: char,
    pub separator_v: char,
    pub ellipsis: &'static str,
    pub arrow_right: &'static str,
    pub arrow_down: &'static str,
}

/// Progress bar characters.
#[derive(Clone, Debug)]
pub struct ProgressGlyphs {
    pub filled: char,
    pub empty: char,
    pub partial: char,
}

/// Chrome/tab bar characters.
#[derive(Clone, Debug)]
pub struct ChromeGlyphs {
    /// Left separator for active tab.
    /// - Nerd: E0B6 (filled left half-circle / rounded left cap)
    /// - Unicode: │
    /// - ASCII: [
    pub tab_left: &'static str,
    /// Right separator for active tab.
    /// - Nerd: E0B4 (filled right half-circle / rounded right cap)
    /// - Unicode: │
    /// - ASCII: ]
    pub tab_right: &'static str,
    /// Separator between inactive tabs (right of active).
    /// - Nerd: E0B1 (powerline thin right arrow)
    /// - Unicode: │
    /// - ASCII: |
    pub tab_separator: &'static str,
    /// Separator between inactive tabs (left of active).
    /// - Nerd: E0B3 (powerline thin left arrow)
    /// - Unicode: │
    /// - ASCII: |
    pub tab_separator_left: &'static str,
    /// Dropdown arrow indicator (shown when multiple tabs exist).
    pub dropdown_arrow: &'static str,
    /// Activity badge: process running / busy.
    pub badge_busy: &'static str,
    /// Activity badge: process idle / waiting for input.
    pub badge_idle: &'static str,
    /// Activity badge: process exited / terminated.
    pub badge_exited: &'static str,
}
