//! CustomPalette — test palette with modified selection color.

use txv_core::palette::dark::DarkPalette;
use txv_core::palette::style_id::StyleId;
use txv_core::prelude::*;

pub(crate) struct CustomPalette;

impl Palette for CustomPalette {
    fn style(&self, id: StyleId) -> Style {
        if id == StyleId::EditSelection {
            Style::default().with_bg(Color::Ansi(5))
        } else {
            DarkPalette.style(id)
        }
    }
}
