//! Dark palette implementation.

use super::style_id::StyleId;
use super::style_palette::StylePalette;
use super::traits::{Base, Chrome, Interactive, Palette, Popup, State};
use super::PaletteStyle;
use crate::cell::{Color, Style};

const fn ansi(n: u8) -> Color {
    Color::Ansi(n)
}

pub struct DarkPalette;

impl StylePalette for DarkPalette {
    fn style(&self, id: StyleId) -> Style {
        match id {
            StyleId::Text => Style::default(),
            StyleId::Dim => PaletteStyle::fg(ansi(8)).to_style(),
            StyleId::Bright => PaletteStyle::fg(ansi(15)).to_style(),
            StyleId::Border => PaletteStyle::new().bold().to_style(),
            StyleId::Separator => PaletteStyle::fg(ansi(8)).to_style(),
            StyleId::TreeDir => PaletteStyle::fg(ansi(14)).to_style(),
            StyleId::CursorFocused => PaletteStyle::bg(ansi(4)).underline().to_style(),
            StyleId::CursorUnfocused => PaletteStyle::bg(ansi(8)).to_style(),
            StyleId::InputCursor => PaletteStyle::colors(ansi(0), ansi(7)).to_style(),
            StyleId::EditOverlay => PaletteStyle::colors(ansi(0), ansi(3)).to_style(),
            StyleId::EditSelection => PaletteStyle::bg(ansi(2)).to_style(),
            StyleId::SearchMatch => PaletteStyle::bg(ansi(3)).to_style(),
            StyleId::VisualSelection => PaletteStyle::colors(ansi(3), ansi(0)).to_style(),
            StyleId::Disabled => PaletteStyle::fg(ansi(8)).to_style(),
            StyleId::ChromeBar => PaletteStyle::colors(ansi(7), ansi(0)).to_style(),
            StyleId::TabFocused => PaletteStyle::colors(ansi(14), ansi(4)).bold().to_style(),
            StyleId::TabFocusedArrow => PaletteStyle::colors(ansi(10), ansi(4)).to_style(),
            StyleId::TabFocusedBadge => PaletteStyle::colors(ansi(15), ansi(6)).bold().to_style(),
            StyleId::TabActive => PaletteStyle::colors(ansi(0), Color::Rgb(0xc0, 0xc0, 0xc0))
                .bold()
                .to_style(),
            StyleId::TabActiveArrow => PaletteStyle::colors(ansi(7), ansi(8)).to_style(),
            StyleId::TabActiveBadge => PaletteStyle::colors(ansi(15), ansi(8)).to_style(),
            StyleId::TabInactive => PaletteStyle::colors(ansi(15), Color::Rgb(0x70, 0x70, 0x70)).to_style(),
            StyleId::StatusBar => PaletteStyle::colors(ansi(7), Color::Palette(236)).to_style(),
            StyleId::StatusBarModal => PaletteStyle::colors(ansi(15), Color::Palette(18)).to_style(),
            StyleId::ScrollbarTrack => PaletteStyle::fg(ansi(8)).to_style(),
            StyleId::ScrollbarThumb => PaletteStyle::colors(ansi(0), ansi(7)).to_style(),
            StyleId::PopupBackground => PaletteStyle::colors(ansi(15), ansi(0)).to_style(),
            StyleId::PopupBorder => PaletteStyle::colors(ansi(6), ansi(0)).to_style(),
            StyleId::PopupSelected => PaletteStyle::colors(ansi(15), ansi(4)).underline().to_style(),
            StyleId::PopupTableHeader => PaletteStyle::colors(ansi(0), ansi(7)).bold().to_style(),
            StyleId::StateError => PaletteStyle::fg(ansi(9)).to_style(),
            StyleId::StateWarning => PaletteStyle::fg(ansi(11)).to_style(),
            StyleId::StateInfo => PaletteStyle::fg(ansi(14)).to_style(),
            StyleId::StateSuccess => PaletteStyle::fg(ansi(2)).to_style(),
            StyleId::StateHint => PaletteStyle::fg(ansi(8)).to_style(),
            StyleId::EditorGutter => PaletteStyle::fg(ansi(8)).to_style(),
        }
    }
}

impl Palette for DarkPalette {
    fn base(&self) -> &dyn Base {
        &DarkBase
    }
    fn interactive(&self) -> &dyn Interactive {
        &DarkInteractive
    }
    fn chrome(&self) -> &dyn Chrome {
        &DarkChrome
    }
    fn popup(&self) -> &dyn Popup {
        &DarkPopup
    }
    fn state(&self) -> &dyn State {
        &DarkState
    }
}

struct DarkBase;
impl Base for DarkBase {
    fn text(&self) -> Style {
        Style::default()
    }
    fn dim(&self) -> Style {
        PaletteStyle::fg(ansi(8)).to_style()
    }
    fn bright(&self) -> Style {
        PaletteStyle::fg(ansi(15)).to_style()
    }
    fn border(&self) -> Style {
        PaletteStyle::new().bold().to_style()
    }
    fn separator(&self) -> Style {
        PaletteStyle::fg(ansi(8)).to_style()
    }
    fn tree_dir(&self) -> Style {
        PaletteStyle::fg(ansi(14)).to_style()
    }
}

struct DarkInteractive;
impl Interactive for DarkInteractive {
    fn cursor_focused(&self) -> Style {
        PaletteStyle::bg(ansi(4)).underline().to_style()
    }
    fn cursor_unfocused(&self) -> Style {
        PaletteStyle::bg(ansi(8)).to_style()
    }
    fn input_cursor(&self) -> Style {
        PaletteStyle::colors(ansi(0), ansi(7)).to_style()
    }
    fn edit_overlay(&self) -> Style {
        PaletteStyle::colors(ansi(0), ansi(3)).to_style()
    }
    fn edit_selection(&self) -> Style {
        PaletteStyle::bg(ansi(2)).to_style()
    }
    fn search_match(&self) -> Style {
        PaletteStyle::bg(ansi(3)).to_style()
    }
    fn visual_selection(&self) -> Style {
        PaletteStyle::colors(ansi(3), ansi(0)).to_style()
    }
    fn disabled(&self) -> Style {
        PaletteStyle::fg(ansi(8)).to_style()
    }
}

struct DarkChrome;
impl Chrome for DarkChrome {
    fn bar(&self) -> Style {
        PaletteStyle::colors(ansi(7), ansi(0)).to_style()
    }
    fn tab_focused(&self) -> Style {
        PaletteStyle::colors(ansi(14), ansi(4)).bold().to_style()
    }
    fn tab_focused_arrow(&self) -> Style {
        PaletteStyle::colors(ansi(10), ansi(4)).to_style()
    }
    fn tab_focused_badge(&self) -> Style {
        PaletteStyle::colors(ansi(15), ansi(6)).bold().to_style()
    }
    fn tab_active(&self) -> Style {
        PaletteStyle::colors(ansi(0), Color::Rgb(0xc0, 0xc0, 0xc0))
            .bold()
            .to_style()
    }
    fn tab_active_arrow(&self) -> Style {
        PaletteStyle::colors(ansi(7), ansi(8)).to_style()
    }
    fn tab_active_badge(&self) -> Style {
        PaletteStyle::colors(ansi(15), ansi(8)).to_style()
    }
    fn tab_inactive(&self, distance: usize) -> Style {
        let gray = (0x70u8).saturating_sub((distance as u8) * 8).max(0x20);
        PaletteStyle::colors(ansi(15), Color::Rgb(gray, gray, gray)).to_style()
    }
    fn status_bar(&self) -> Style {
        PaletteStyle::colors(ansi(7), Color::Palette(236)).to_style()
    }
    fn status_bar_modal(&self) -> Style {
        PaletteStyle::colors(ansi(15), Color::Palette(18)).to_style()
    }
    fn scrollbar_track(&self) -> Style {
        PaletteStyle::fg(ansi(8)).to_style()
    }
    fn scrollbar_thumb(&self) -> Style {
        PaletteStyle::colors(ansi(0), ansi(7)).to_style()
    }
}

struct DarkPopup;
impl Popup for DarkPopup {
    fn background(&self) -> Style {
        PaletteStyle::colors(ansi(15), ansi(0)).to_style()
    }
    fn border(&self) -> Style {
        PaletteStyle::colors(ansi(6), ansi(0)).to_style()
    }
    fn selected(&self) -> Style {
        PaletteStyle::colors(ansi(15), ansi(4)).underline().to_style()
    }
    fn table_header(&self) -> Style {
        PaletteStyle::colors(ansi(0), ansi(7)).bold().to_style()
    }
}

pub(super) struct DarkState;
impl State for DarkState {
    fn error(&self) -> Style {
        PaletteStyle::fg(ansi(9)).to_style()
    }
    fn warning(&self) -> Style {
        PaletteStyle::fg(ansi(11)).to_style()
    }
    fn info(&self) -> Style {
        PaletteStyle::fg(ansi(14)).to_style()
    }
    fn success(&self) -> Style {
        PaletteStyle::fg(ansi(2)).to_style()
    }
    fn hint(&self) -> Style {
        PaletteStyle::fg(ansi(8)).to_style()
    }
}
