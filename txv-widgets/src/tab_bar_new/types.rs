//! Types for TabBar configuration.

use txv_core::prelude::*;

/// Tab bar display mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabBarMode {
    /// Only active tab name shown + `▾N──` badge. Current kairn style.
    Single,
    /// All tabs visible, fixed positions, all numbered: `₁Files ₂Git ₃Tools`.
    Static,
    /// Active leftmost (no number), rest by recency: `mod.rs ₁buf.rs ▾…3`.
    Lru,
}

/// Style for a single tab position in the palette.
#[derive(Clone, Copy, Debug)]
pub struct TabStyle {
    pub fg: Color,
    pub bg: Color,
}

impl Default for TabStyle {
    fn default() -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reset,
        }
    }
}

/// Color palette for the tab bar. Positional lookup.
#[derive(Clone, Debug)]
pub struct TabBarPalette {
    /// Active tab when panel is focused.
    pub active_focused: TabStyle,
    /// Active tab when panel is not focused.
    pub active_unfocused: TabStyle,
    /// Positional styles for inactive tabs (index 0 = nearest to active).
    pub inactive: [TabStyle; 10],
    /// Dim foreground for thin separators and fill.
    pub dim_fg: Color,
    /// Badge bg when panel is focused.
    pub badge_focused_bg: Color,
    /// Badge fg.
    pub badge_fg: Color,
    /// Separator fg between inactive tabs.
    pub separator_fg: Color,
}

impl TabBarPalette {
    /// Derive tab bar palette from the global palette.
    pub fn from_global_palette() -> Self {
        use txv_core::palette::palette;
        let pal = palette();
        let focused = pal.chrome().tab_focused();
        let active = pal.chrome().tab_active();
        let dim_fg = pal.base().dim().fg;
        let badge = pal.chrome().tab_focused_badge();
        let separator_fg = pal.base().text().bg;

        // Read gradient from palette
        let mut inactive = [TabStyle::default(); 10];
        for (i, s) in inactive.iter_mut().enumerate() {
            let style = pal.chrome().tab_inactive(i);
            s.fg = style.fg;
            s.bg = style.bg;
        }
        Self {
            active_focused: TabStyle {
                fg: focused.fg,
                bg: focused.bg,
            },
            active_unfocused: TabStyle {
                fg: active.fg,
                bg: active.bg,
            },
            inactive,
            dim_fg,
            badge_focused_bg: badge.bg,
            badge_fg: badge.fg,
            separator_fg,
        }
    }
}

impl Default for TabBarPalette {
    fn default() -> Self {
        Self::from_global_palette()
    }
}

/// Fill style for the bar area not covered by tabs.
#[derive(Clone, Copy, Debug)]
pub struct TabBarFill {
    pub ch: char,
    pub style: Style,
}

impl TabBarFill {
    /// Transparent fill — parent's content shows through.
    pub fn transparent() -> Self {
        Self {
            ch: ' ',
            style: Style {
                fg: Color::Transparent,
                bg: Color::Transparent,
                ..Style::default()
            },
        }
    }

    /// Horizontal rule fill.
    pub fn rule(fg: Color) -> Self {
        Self {
            ch: '─',
            style: Style {
                fg,
                bg: Color::Reset,
                ..Style::default()
            },
        }
    }
}

impl Default for TabBarFill {
    fn default() -> Self {
        Self::transparent()
    }
}

/// Subscript digits for tab numbering.
pub(crate) const SUBSCRIPTS: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];
