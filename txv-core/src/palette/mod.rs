//! Palette — semantic style lookup by StyleId.
//!
//! Views call `palette().style(StyleId::StatusBar)` to get a Style.
//! Implementations (dark/light) are swappable at runtime.

pub mod dark;
mod derived_palette;
pub mod light;
mod palette_style;
pub mod style_id;

#[cfg(test)]
mod tests;

use std::sync::{Arc, OnceLock, RwLock};

use crate::cell::Style;

pub use derived_palette::DerivedPalette;
pub use palette_style::PaletteStyle;
pub use style_id::StyleId;

/// A palette maps StyleId → Style. Views receive a palette from their parent
/// or use the global palette.
pub trait Palette: Send + Sync {
    fn style(&self, id: StyleId) -> Style;

    /// Tab inactive style with distance-based gradient. Default darkens bg by distance.
    fn tab_inactive(&self, distance: usize) -> Style {
        let base = self.style(StyleId::TabInactive);
        let darken = |c: crate::cell::Color| match c {
            crate::cell::Color::Rgb(r, g, b) => {
                let d = (distance as u8).saturating_mul(8);
                crate::cell::Color::Rgb(r.saturating_sub(d), g.saturating_sub(d), b.saturating_sub(d))
            }
            _ => c,
        };
        Style {
            bg: darken(base.bg),
            ..base
        }
    }
}

static PALETTE: OnceLock<RwLock<Arc<dyn Palette>>> = OnceLock::new();

/// Get the active palette.
pub fn palette() -> Arc<dyn Palette> {
    match PALETTE.get() {
        Some(lock) => lock
            .read()
            .map(|p| Arc::clone(&p))
            .unwrap_or_else(|_| Arc::new(dark::DarkPalette)),
        None => Arc::new(dark::DarkPalette),
    }
}

/// Set the active palette (call on startup and on theme toggle).
pub fn set_palette(p: Arc<dyn Palette>) {
    match PALETTE.get() {
        Some(lock) => {
            if let Ok(mut w) = lock.write() {
                *w = p;
            }
        }
        None => {
            let _ = PALETTE.set(RwLock::new(p));
        }
    }
}

/// Theme mode: dark, light, or auto-detect.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
    Auto,
}

/// Detect system theme preference.
pub fn detect_system_theme() -> ThemeMode {
    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = std::process::Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout);
            if s.trim().eq_ignore_ascii_case("dark") {
                return ThemeMode::Dark;
            }
            if out.status.success() {
                return ThemeMode::Light;
            }
        }
    }
    if let Ok(val) = std::env::var("COLORFGBG") {
        if let Some(bg) = val.rsplit(';').next().and_then(|s| s.parse::<u8>().ok()) {
            return if bg < 8 {
                ThemeMode::Dark
            } else {
                ThemeMode::Light
            };
        }
    }
    ThemeMode::Dark
}
