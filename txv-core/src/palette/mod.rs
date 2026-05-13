//! Color palette system — semantic style roles with dark/light mode support.
//!
//! Views call `palette()` to get the active palette. Set once at startup,
//! swappable at runtime via `set_palette()` for dark/light toggle.

mod defaults;
mod defs;

use std::sync::OnceLock;

use crate::cell::{Attrs, Color, Style};

pub use defs::{BasePalette, ChromePalette, InteractivePalette, Palette, PopupPalette, StatePalette};

static PALETTE: OnceLock<std::sync::RwLock<Palette>> = OnceLock::new();

/// Get the active palette.
pub fn palette() -> Palette {
    match PALETTE.get() {
        Some(lock) => lock.read().map(|p| p.clone()).unwrap_or_default(),
        None => Palette::default(),
    }
}

/// Set the active palette (call on startup and on theme toggle).
pub fn set_palette(p: Palette) {
    match PALETTE.get() {
        Some(lock) => {
            if let Ok(mut w) = lock.write() {
                *w = p;
            }
        }
        None => {
            let _ = PALETTE.set(std::sync::RwLock::new(p));
        }
    }
}

/// Theme mode: dark, light, or auto-detect.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Dark,
    Light,
    Auto,
}

/// Detect system theme preference.
/// Falls back to Dark if detection fails.
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

/// A single palette entry. Option fields support partial override.
#[derive(Clone, Debug, Default)]
pub struct PaletteStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub attrs: Option<Attrs>,
}

impl PaletteStyle {
    pub const fn fg(color: Color) -> Self {
        Self {
            fg: Some(color),
            bg: None,
            attrs: None,
        }
    }
    pub const fn bg(color: Color) -> Self {
        Self {
            fg: None,
            bg: Some(color),
            attrs: None,
        }
    }
    pub const fn colors(fg: Color, bg: Color) -> Self {
        Self {
            fg: Some(fg),
            bg: Some(bg),
            attrs: None,
        }
    }

    /// Resolve to concrete Style, filling unset fields from `base`.
    pub fn resolve(&self, base: &Style) -> Style {
        Style {
            fg: self.fg.unwrap_or(base.fg),
            bg: self.bg.unwrap_or(base.bg),
            attrs: self.attrs.unwrap_or(base.attrs),
        }
    }

    /// Resolve using default Style (Reset/Reset/no attrs).
    pub fn to_style(&self) -> Style {
        self.resolve(&Style::default())
    }

    /// Merge overlay on top of self (overlay wins where set).
    pub fn merge(&self, overlay: &PaletteStyle) -> PaletteStyle {
        PaletteStyle {
            fg: overlay.fg.or(self.fg),
            bg: overlay.bg.or(self.bg),
            attrs: overlay.attrs.or(self.attrs),
        }
    }
}
