//! Default glyph sets for each tier.

use super::defs::*;

impl Default for GlyphSet {
    fn default() -> Self {
        Self::nerd()
    }
}

impl GlyphSet {
    pub fn ascii() -> Self {
        Self {
            tier: GlyphTier::Ascii,
            box_drawing: BoxGlyphs::ascii(),
            tree: TreeGlyphs::ascii(),
            ui: UiGlyphs::ascii(),
            chrome: ChromeGlyphs::ascii(),
            progress: ProgressGlyphs::ascii(),
        }
    }

    pub fn unicode() -> Self {
        Self {
            tier: GlyphTier::Unicode,
            box_drawing: BoxGlyphs::unicode(),
            tree: TreeGlyphs::unicode(),
            ui: UiGlyphs::unicode(),
            chrome: ChromeGlyphs::unicode(),
            progress: ProgressGlyphs::unicode(),
        }
    }

    pub fn unicode_extended() -> Self {
        Self {
            tier: GlyphTier::UnicodeExtended,
            box_drawing: BoxGlyphs::unicode_extended(),
            tree: TreeGlyphs::unicode(),
            ui: UiGlyphs::unicode(),
            chrome: ChromeGlyphs::unicode(),
            progress: ProgressGlyphs::unicode(),
        }
    }

    pub fn nerd() -> Self {
        Self {
            tier: GlyphTier::Nerd,
            box_drawing: BoxGlyphs::unicode_extended(),
            tree: TreeGlyphs::nerd(),
            ui: UiGlyphs::nerd(),
            chrome: ChromeGlyphs::nerd(),
            progress: ProgressGlyphs::unicode(),
        }
    }

    /// Create from tier enum.
    pub fn from_tier(tier: GlyphTier) -> Self {
        match tier {
            GlyphTier::Ascii => Self::ascii(),
            GlyphTier::Unicode => Self::unicode(),
            GlyphTier::UnicodeExtended => Self::unicode_extended(),
            GlyphTier::Nerd => Self::nerd(),
        }
    }
}

impl BoxGlyphs {
    pub fn ascii() -> Self {
        Self {
            h: '-',
            v: '|',
            tl: '+',
            tr: '+',
            bl: '+',
            br: '+',
            h_heavy: '=',
            v_heavy: '|',
            tl_heavy: '+',
            tr_heavy: '+',
            bl_heavy: '+',
            br_heavy: '+',
            tl_round: '+',
            tr_round: '+',
            bl_round: '+',
            br_round: '+',
        }
    }

    pub fn unicode() -> Self {
        Self {
            h: '─',
            v: '│',
            tl: '┌',
            tr: '┐',
            bl: '└',
            br: '┘',
            h_heavy: '═',
            v_heavy: '║',
            tl_heavy: '╔',
            tr_heavy: '╗',
            bl_heavy: '╚',
            br_heavy: '╝',
            tl_round: '┌',
            tr_round: '┐',
            bl_round: '└',
            br_round: '┘',
        }
    }

    pub fn unicode_extended() -> Self {
        Self {
            tl_round: '╭',
            tr_round: '╮',
            bl_round: '╰',
            br_round: '╯',
            ..Self::unicode()
        }
    }
}

impl TreeGlyphs {
    pub fn ascii() -> Self {
        Self {
            expanded: "v ",
            collapsed: "> ",
            branch: '+',
            last_branch: '`',
            pipe: '|',
            open_indicator: "*",
        }
    }

    pub fn unicode() -> Self {
        Self {
            expanded: "▼ ",
            collapsed: "▶ ",
            branch: '├',
            last_branch: '└',
            pipe: '│',
            open_indicator: "◉",
        }
    }

    pub fn nerd() -> Self {
        Self {
            expanded: "\u{F0D7} ",
            collapsed: "\u{F0DA} ",
            branch: '├',
            last_branch: '└',
            pipe: '│',
            open_indicator: "\u{F06E}",
        }
    }
}

impl UiGlyphs {
    pub fn ascii() -> Self {
        Self {
            scrollbar_track: '|',
            scrollbar_thumb: '#',
            separator_h: '-',
            separator_v: '|',
            ellipsis: "...",
            arrow_right: ">",
            arrow_down: "v",
        }
    }

    pub fn unicode() -> Self {
        Self {
            scrollbar_track: '│',
            scrollbar_thumb: '█',
            separator_h: '─',
            separator_v: '│',
            ellipsis: "…",
            arrow_right: "▸",
            arrow_down: "▾",
        }
    }

    pub fn nerd() -> Self {
        Self {
            scrollbar_track: '│',
            scrollbar_thumb: '█',
            separator_h: '─',
            separator_v: '│',
            ellipsis: "…",
            arrow_right: "\u{E0B1}", // Powerline thin separator
            arrow_down: "▾",
        }
    }
}

impl ProgressGlyphs {
    pub fn ascii() -> Self {
        Self {
            filled: '#',
            empty: '.',
            partial: '-',
        }
    }

    pub fn unicode() -> Self {
        Self {
            filled: '█',
            empty: '░',
            partial: '▒',
        }
    }
}

impl ChromeGlyphs {
    pub fn ascii() -> Self {
        Self {
            tab_left: "[",
            tab_right: "]",
            tab_separator: "|",
            tab_separator_left: "|",
            dropdown_arrow: "v",
            badge_busy: "*",
            badge_idle: "o",
            badge_exited: "x",
        }
    }

    pub fn unicode() -> Self {
        Self {
            tab_left: "│",
            tab_right: "│",
            tab_separator: "│",
            tab_separator_left: "│",
            dropdown_arrow: "▾",
            badge_busy: "◉",
            badge_idle: "●",
            badge_exited: "✗",
        }
    }

    pub fn nerd() -> Self {
        Self {
            tab_left: "\u{E0B6}",            // Powerline left half-circle
            tab_right: "\u{E0B4}",           // Powerline right half-circle
            tab_separator: " \u{E0B1}",      // space + Powerline thin right arrow
            tab_separator_left: "\u{E0B3} ", // Powerline thin left arrow + space
            dropdown_arrow: "▾",
            badge_busy: "◉",
            badge_idle: "●",
            badge_exited: "✗",
        }
    }
}
