//! Light palette implementation.

use super::dark::DarkState;
use super::traits::{Base, Chrome, Interactive, Palette, Popup, State};
use super::PaletteStyle;
use crate::cell::{Color, Style};

const fn ansi(n: u8) -> Color {
    Color::Ansi(n)
}

pub struct LightPalette;

impl Palette for LightPalette {
    fn base(&self) -> &dyn Base {
        &LightBase
    }
    fn interactive(&self) -> &dyn Interactive {
        &LightInteractive
    }
    fn chrome(&self) -> &dyn Chrome {
        &LightChrome
    }
    fn popup(&self) -> &dyn Popup {
        &LightPopup
    }
    fn state(&self) -> &dyn State {
        &DarkState
    }
}

struct LightBase;
impl Base for LightBase {
    fn text(&self) -> Style {
        Style::default()
    }
    fn dim(&self) -> Style {
        PaletteStyle::fg(ansi(7)).to_style()
    }
    fn bright(&self) -> Style {
        PaletteStyle::fg(ansi(0)).to_style()
    }
    fn border(&self) -> Style {
        PaletteStyle::new().bold().to_style()
    }
    fn separator(&self) -> Style {
        PaletteStyle::fg(ansi(7)).to_style()
    }
    fn tree_dir(&self) -> Style {
        PaletteStyle::fg(ansi(4)).to_style()
    }
}

struct LightInteractive;
impl Interactive for LightInteractive {
    fn cursor_focused(&self) -> Style {
        PaletteStyle::bg(ansi(12)).underline().to_style()
    }
    fn cursor_unfocused(&self) -> Style {
        PaletteStyle::bg(ansi(7)).to_style()
    }
    fn input_cursor(&self) -> Style {
        PaletteStyle::colors(ansi(15), ansi(0)).to_style()
    }
    fn edit_overlay(&self) -> Style {
        PaletteStyle::colors(ansi(0), ansi(11)).to_style()
    }
    fn edit_selection(&self) -> Style {
        PaletteStyle::bg(ansi(10)).to_style()
    }
    fn search_match(&self) -> Style {
        PaletteStyle::bg(ansi(11)).to_style()
    }
    fn visual_selection(&self) -> Style {
        PaletteStyle::colors(ansi(4), ansi(15)).to_style()
    }
    fn disabled(&self) -> Style {
        PaletteStyle::fg(ansi(7)).to_style()
    }
}

struct LightChrome;
impl Chrome for LightChrome {
    fn bar(&self) -> Style {
        PaletteStyle::colors(ansi(0), ansi(7)).to_style()
    }
    fn tab_focused(&self) -> Style {
        PaletteStyle::colors(ansi(4), ansi(12)).bold().to_style()
    }
    fn tab_focused_arrow(&self) -> Style {
        PaletteStyle::colors(ansi(4), ansi(12)).to_style()
    }
    fn tab_focused_badge(&self) -> Style {
        PaletteStyle::colors(ansi(15), ansi(4)).bold().to_style()
    }
    fn tab_active(&self) -> Style {
        PaletteStyle::colors(ansi(0), ansi(7)).bold().to_style()
    }
    fn tab_active_arrow(&self) -> Style {
        PaletteStyle::colors(ansi(0), ansi(7)).to_style()
    }
    fn tab_active_badge(&self) -> Style {
        PaletteStyle::colors(ansi(0), ansi(7)).to_style()
    }
    fn tab_inactive(&self, distance: usize) -> Style {
        let gray = (0xd0u8).saturating_sub((distance as u8) * 8).max(0x90);
        PaletteStyle::colors(ansi(0), Color::Rgb(gray, gray, gray)).to_style()
    }
    fn status_bar(&self) -> Style {
        PaletteStyle::colors(ansi(0), ansi(7)).to_style()
    }
    fn scrollbar_track(&self) -> Style {
        PaletteStyle::fg(ansi(7)).to_style()
    }
    fn scrollbar_thumb(&self) -> Style {
        PaletteStyle::colors(ansi(15), ansi(0)).to_style()
    }
}

struct LightPopup;
impl Popup for LightPopup {
    fn background(&self) -> Style {
        PaletteStyle::colors(ansi(0), ansi(15)).to_style()
    }
    fn border(&self) -> Style {
        PaletteStyle::colors(ansi(4), ansi(15)).to_style()
    }
    fn selected(&self) -> Style {
        PaletteStyle::colors(ansi(15), ansi(4)).underline().to_style()
    }
    fn table_header(&self) -> Style {
        PaletteStyle::colors(ansi(15), ansi(0)).bold().to_style()
    }
}
