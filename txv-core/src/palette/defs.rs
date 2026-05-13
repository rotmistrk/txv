//! Palette struct definitions.

use super::PaletteStyle;
use crate::cell::{Attrs, Color};

/// Framework-level palette — roles that any txv app needs.
#[derive(Clone, Debug, Default)]
pub struct Palette {
    pub base: BasePalette,
    pub interactive: InteractivePalette,
    pub chrome: ChromePalette,
    pub popup: PopupPalette,
    pub state: StatePalette,
}

#[derive(Clone, Debug)]
pub struct BasePalette {
    pub text: PaletteStyle,
    pub dim: PaletteStyle,
    pub bright: PaletteStyle,
    pub border: PaletteStyle,
    pub separator: PaletteStyle,
}

#[derive(Clone, Debug)]
pub struct InteractivePalette {
    pub cursor_focused: PaletteStyle,
    pub cursor_unfocused: PaletteStyle,
    pub input_cursor: PaletteStyle,
    pub edit_overlay: PaletteStyle,
    pub search_match: PaletteStyle,
    pub visual_selection: PaletteStyle,
    pub disabled: PaletteStyle,
}

#[derive(Clone, Debug)]
pub struct ChromePalette {
    pub bar: PaletteStyle,
    pub tab_focused: PaletteStyle,
    pub tab_focused_arrow: PaletteStyle,
    pub tab_focused_badge: PaletteStyle,
    pub tab_active: PaletteStyle,
    pub tab_active_arrow: PaletteStyle,
    pub tab_active_badge: PaletteStyle,
    pub status_bar: PaletteStyle,
    pub scrollbar_track: PaletteStyle,
    pub scrollbar_thumb: PaletteStyle,
}

#[derive(Clone, Debug)]
pub struct PopupPalette {
    pub background: PaletteStyle,
    pub border: PaletteStyle,
    pub selected: PaletteStyle,
    pub table_header: PaletteStyle,
}

#[derive(Clone, Debug)]
pub struct StatePalette {
    pub error: PaletteStyle,
    pub warning: PaletteStyle,
    pub info: PaletteStyle,
    pub success: PaletteStyle,
    pub hint: PaletteStyle,
}

impl Palette {
    /// Dark theme defaults (current behavior).
    pub fn dark() -> Self {
        Self::default()
    }

    /// Light theme defaults.
    pub fn light() -> Self {
        let mut p = Self::default();
        p.base.dim = PaletteStyle::fg(Color::Ansi(7));
        p.interactive.cursor_focused = PaletteStyle {
            bg: Some(Color::Ansi(12)),
            attrs: Some(Attrs {
                underline: true,
                ..Attrs::default()
            }),
            ..Default::default()
        };
        p.interactive.cursor_unfocused = PaletteStyle::bg(Color::Ansi(7));
        p.interactive.edit_overlay = PaletteStyle::colors(Color::Ansi(0), Color::Ansi(11));
        p.chrome.bar = PaletteStyle::colors(Color::Ansi(0), Color::Ansi(7));
        p.chrome.tab_focused = PaletteStyle {
            fg: Some(Color::Ansi(4)),
            bg: Some(Color::Ansi(12)),
            attrs: Some(Attrs {
                bold: true,
                ..Attrs::default()
            }),
        };
        p.chrome.tab_active = PaletteStyle {
            fg: Some(Color::Ansi(0)),
            bg: Some(Color::Ansi(7)),
            attrs: Some(Attrs {
                bold: true,
                ..Attrs::default()
            }),
        };
        p.popup.background = PaletteStyle::colors(Color::Ansi(0), Color::Ansi(15));
        p.popup.border = PaletteStyle::colors(Color::Ansi(4), Color::Ansi(15));
        p
    }
}
