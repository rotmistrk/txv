//! TabStyle — style for a single tab position.

use txv_core::prelude::*;

/// Style for a single tab position in the palette.
#[derive(Clone, Copy, Debug)]
pub struct TabStyle {
    pub(crate) fg: Color,
    pub(crate) bg: Color,
}

impl TabStyle {
    pub fn bg(&self) -> Color {
        self.bg
    }
}

impl Default for TabStyle {
    fn default() -> Self {
        Self {
            fg: Color::Reset,
            bg: Color::Reset,
        }
    }
}
