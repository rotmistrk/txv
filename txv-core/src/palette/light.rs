//! Light palette implementation.

use super::style_id::StyleId;
use super::{Palette, PaletteStyle};
use crate::cell::{Color, Style};

const fn ansi(n: u8) -> Color {
    Color::Ansi(n)
}

pub struct LightPalette;

impl Palette for LightPalette {
    fn style(&self, id: StyleId) -> Style {
        match id {
            StyleId::Text => Style::default(),
            StyleId::Dim => PaletteStyle::fg(ansi(8)).to_style(),
            StyleId::Bright => PaletteStyle::fg(ansi(0)).to_style(),
            StyleId::Border => PaletteStyle::fg(ansi(8)).bold().to_style(),
            StyleId::Separator => PaletteStyle::fg(ansi(7)).to_style(),
            StyleId::TreeDir => PaletteStyle::fg(ansi(4)).to_style(),
            StyleId::CursorFocused => PaletteStyle::colors(ansi(0), ansi(12)).underline().to_style(),
            StyleId::CursorUnfocused => PaletteStyle::bg(Color::Rgb(0xe0, 0xe0, 0xe0)).to_style(),
            StyleId::InputCursor => PaletteStyle::colors(ansi(15), ansi(0)).to_style(),
            StyleId::EditSelection => PaletteStyle::bg(Color::Rgb(0xad, 0xd8, 0xe6)).to_style(),
            StyleId::OverflowIndicator => PaletteStyle::fg(ansi(1)).to_style(),
            StyleId::SearchMatch => PaletteStyle::bg(Color::Rgb(0xff, 0xff, 0x80)).to_style(),
            StyleId::VisualSelection => PaletteStyle::colors(ansi(0), Color::Rgb(0xad, 0xd8, 0xe6)).to_style(),
            StyleId::Disabled => PaletteStyle::fg(ansi(7)).to_style(),
            StyleId::ChromeBar => PaletteStyle::colors(ansi(0), Color::Rgb(0xe8, 0xe8, 0xe8)).to_style(),
            StyleId::TabFocused => PaletteStyle::colors(ansi(4), Color::Rgb(0xd0, 0xe8, 0xff))
                .bold()
                .to_style(),
            StyleId::TabFocusedArrow => PaletteStyle::colors(ansi(4), Color::Rgb(0xd0, 0xe8, 0xff)).to_style(),
            StyleId::TabFocusedBadge => PaletteStyle::colors(ansi(15), ansi(4)).bold().to_style(),
            _ => light_remaining(id),
        }
    }
}

fn light_remaining(id: StyleId) -> Style {
    match id {
        StyleId::TabActive => PaletteStyle::colors(ansi(0), Color::Rgb(0xe8, 0xe8, 0xe8))
            .bold()
            .to_style(),
        StyleId::TabActiveArrow => PaletteStyle::colors(ansi(8), Color::Rgb(0xd0, 0xd0, 0xd0)).to_style(),
        StyleId::TabActiveBadge => PaletteStyle::colors(ansi(0), Color::Rgb(0xd0, 0xd0, 0xd0)).to_style(),
        StyleId::TabInactive => PaletteStyle::colors(ansi(8), Color::Rgb(0xf0, 0xf0, 0xf0)).to_style(),
        StyleId::StatusBar => PaletteStyle::colors(ansi(0), Color::Rgb(0xe8, 0xe8, 0xe8)).to_style(),
        StyleId::StatusBarModal => PaletteStyle::colors(ansi(15), ansi(4)).to_style(),
        StyleId::ScrollbarTrack => PaletteStyle::fg(Color::Rgb(0xd0, 0xd0, 0xd0)).to_style(),
        StyleId::ScrollbarThumb => PaletteStyle::colors(ansi(15), ansi(8)).to_style(),
        StyleId::PopupBackground => PaletteStyle::colors(ansi(0), ansi(15)).to_style(),
        StyleId::PopupBorder => PaletteStyle::colors(ansi(4), ansi(15)).to_style(),
        StyleId::PopupSelected => PaletteStyle::colors(ansi(0), Color::Rgb(0xd0, 0xe8, 0xff))
            .underline()
            .to_style(),
        StyleId::PopupTableHeader => PaletteStyle::colors(ansi(15), ansi(4)).bold().to_style(),
        StyleId::StateError => PaletteStyle::fg(ansi(1)).to_style(),
        StyleId::StateWarning => PaletteStyle::fg(ansi(3)).to_style(),
        StyleId::StateInfo => PaletteStyle::fg(ansi(6)).to_style(),
        StyleId::StateSuccess => PaletteStyle::fg(ansi(2)).to_style(),
        StyleId::StateHint => PaletteStyle::fg(ansi(8)).to_style(),
        StyleId::EditorGutter => PaletteStyle::fg(ansi(8)).to_style(),
        StyleId::StatusQuestion => PaletteStyle::fg(ansi(5)).bold().to_style(),
        StyleId::StatusHighlight => PaletteStyle::fg(ansi(0)).bold().to_style(),
        StyleId::TableRowActive => PaletteStyle::new().underline().to_style(),
        StyleId::TableRowInactive => PaletteStyle::fg(ansi(8)).underline().to_style(),
        StyleId::TreeGuide => PaletteStyle::fg(Color::Rgb(0xc0, 0xc0, 0xc0)).to_style(),
        StyleId::DropdownNumber => PaletteStyle::fg(ansi(4)).to_style(),
        StyleId::TabNumber => PaletteStyle::fg(ansi(4)).to_style(),
        _ => Style::default(),
    }
}
