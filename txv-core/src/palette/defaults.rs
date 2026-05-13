//! Default implementations for palette sub-structs.

use super::defs::{BasePalette, ChromePalette, InteractivePalette, PopupPalette, StatePalette};
use super::PaletteStyle;
use crate::cell::{Attrs, Color};

const fn ansi(n: u8) -> Color {
    Color::Ansi(n)
}
const fn ps_fg(n: u8) -> PaletteStyle {
    PaletteStyle::fg(ansi(n))
}
const fn ps_bg(n: u8) -> PaletteStyle {
    PaletteStyle::bg(ansi(n))
}
const fn ps_fgbg(f: u8, b: u8) -> PaletteStyle {
    PaletteStyle::colors(ansi(f), ansi(b))
}

fn attrs_bold() -> Attrs {
    Attrs {
        bold: true,
        ..Attrs::default()
    }
}
fn attrs_underline() -> Attrs {
    Attrs {
        underline: true,
        ..Attrs::default()
    }
}
fn attrs_reverse() -> Attrs {
    Attrs {
        reverse: true,
        ..Attrs::default()
    }
}
fn attrs_bold_reverse() -> Attrs {
    Attrs {
        bold: true,
        reverse: true,
        ..Attrs::default()
    }
}

impl Default for BasePalette {
    fn default() -> Self {
        Self {
            text: PaletteStyle::default(),
            dim: ps_fg(8),
            bright: ps_fg(15),
            border: PaletteStyle {
                attrs: Some(attrs_bold()),
                ..Default::default()
            },
            separator: ps_fg(8),
        }
    }
}

impl Default for InteractivePalette {
    fn default() -> Self {
        Self {
            cursor_focused: PaletteStyle {
                bg: Some(ansi(4)),
                attrs: Some(attrs_underline()),
                ..Default::default()
            },
            cursor_unfocused: ps_bg(8),
            input_cursor: PaletteStyle {
                attrs: Some(attrs_reverse()),
                ..Default::default()
            },
            edit_overlay: ps_fgbg(0, 3),
            search_match: ps_bg(3),
            visual_selection: PaletteStyle {
                fg: Some(ansi(3)),
                attrs: Some(attrs_reverse()),
                ..Default::default()
            },
            disabled: ps_fg(8),
        }
    }
}

impl Default for ChromePalette {
    fn default() -> Self {
        Self {
            bar: ps_fgbg(7, 0),
            tab_focused: PaletteStyle {
                fg: Some(ansi(14)),
                bg: Some(ansi(4)),
                attrs: Some(attrs_bold()),
            },
            tab_focused_arrow: ps_fgbg(10, 4),
            tab_focused_badge: PaletteStyle {
                fg: Some(ansi(15)),
                bg: Some(ansi(6)),
                attrs: Some(attrs_bold()),
            },
            tab_active: PaletteStyle {
                fg: Some(ansi(15)),
                bg: Some(ansi(8)),
                attrs: Some(attrs_bold()),
            },
            tab_active_arrow: ps_fgbg(7, 8),
            tab_active_badge: ps_fgbg(15, 8),
            status_bar: PaletteStyle {
                attrs: Some(attrs_reverse()),
                ..Default::default()
            },
            scrollbar_track: ps_fg(8),
            scrollbar_thumb: PaletteStyle {
                attrs: Some(attrs_reverse()),
                ..Default::default()
            },
        }
    }
}

impl Default for PopupPalette {
    fn default() -> Self {
        Self {
            background: ps_fgbg(15, 0),
            border: ps_fgbg(6, 0),
            selected: PaletteStyle {
                fg: Some(ansi(15)),
                bg: Some(ansi(4)),
                attrs: Some(attrs_underline()),
            },
            table_header: PaletteStyle {
                attrs: Some(attrs_bold_reverse()),
                ..Default::default()
            },
        }
    }
}

impl Default for StatePalette {
    fn default() -> Self {
        Self {
            error: ps_fg(1),
            warning: ps_fg(3),
            info: ps_fg(6),
            success: ps_fg(2),
            hint: ps_fg(8),
        }
    }
}
