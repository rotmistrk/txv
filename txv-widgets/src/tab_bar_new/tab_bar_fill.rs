//! TabBarFill — fill style for the bar area not covered by tabs.

use txv_core::prelude::*;

/// Fill style for the bar area not covered by tabs.
#[derive(Clone, Copy, Debug)]
pub struct TabBarFill {
    pub(crate) ch: char,
    pub(crate) style: Style,
}

impl TabBarFill {
    /// Transparent fill — parent's content shows through.
    pub fn transparent() -> Self {
        Self {
            ch: ' ',
            style: Style::new(Color::Transparent, Color::Transparent),
        }
    }

    /// Horizontal rule fill.
    pub fn rule(fg: Color) -> Self {
        Self {
            ch: '─',
            style: Style::new(fg, Color::Reset),
        }
    }
}

impl Default for TabBarFill {
    fn default() -> Self {
        Self::transparent()
    }
}
