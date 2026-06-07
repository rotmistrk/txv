//! TabBarPalette — color palette for the tab bar.

use txv_core::prelude::*;

use super::tab_style::TabStyle;

/// Color palette for the tab bar. Positional lookup.
#[derive(Clone, Debug)]
pub struct TabBarPalette {
    /// Active tab when panel is focused.
    pub(crate) active_focused: TabStyle,
    /// Active tab when panel is not focused.
    pub(crate) active_unfocused: TabStyle,
    /// Positional styles for inactive tabs (index 0 = nearest to active).
    pub(crate) inactive: [TabStyle; 10],
    /// Dim foreground for thin separators and fill.
    pub(crate) dim_fg: Color,
    /// Badge bg when panel is focused.
    pub(crate) badge_focused_bg: Color,
    /// Badge fg.
    pub(crate) badge_fg: Color,
    /// Separator fg between inactive tabs.
    pub(crate) separator_fg: Color,
}

impl TabBarPalette {
    /// Derive tab bar palette from the global palette.
    pub fn from_global_palette() -> Self {
        use txv_core::palette::palette;
        let pal = palette();
        let focused = pal.style(StyleId::TabFocused);
        let active = pal.style(StyleId::TabActive);
        let dim_fg = pal.style(StyleId::Dim).fg();
        let badge = pal.style(StyleId::TabFocusedBadge);
        let separator_fg = pal.style(StyleId::Text).bg();

        // Read gradient from palette
        let mut inactive = [TabStyle::default(); 10];
        for (i, s) in inactive.iter_mut().enumerate() {
            let style = pal.tab_inactive(i);
            s.fg = style.fg();
            s.bg = style.bg();
        }
        Self {
            active_focused: TabStyle {
                fg: focused.fg(),
                bg: focused.bg(),
            },
            active_unfocused: TabStyle {
                fg: active.fg(),
                bg: active.bg(),
            },
            inactive,
            dim_fg,
            badge_focused_bg: badge.bg(),
            badge_fg: badge.fg(),
            separator_fg,
        }
    }
}

impl Default for TabBarPalette {
    fn default() -> Self {
        Self::from_global_palette()
    }
}
