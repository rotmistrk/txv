//! Palette — semantic style lookup by StyleId.
//!
//! Views call `palette().style(StyleId::StatusBar)` to get a Style.
//! Implementations (dark/light) are swappable at runtime.

pub mod dark;
pub mod light;
pub mod style_id;

#[cfg(test)]
mod tests;

use std::sync::{Arc, OnceLock, RwLock};

use crate::cell::Style;

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

/// A palette that wraps another and overrides specific style IDs.
pub struct DerivedPalette {
    base: Arc<dyn Palette>,
    overrides: Vec<(StyleId, Style)>,
}

impl DerivedPalette {
    pub fn new(base: Arc<dyn Palette>) -> Self {
        Self {
            base,
            overrides: Vec::new(),
        }
    }

    pub fn with_override(mut self, id: StyleId, style: Style) -> Self {
        self.overrides.push((id, style));
        self
    }
}

impl Palette for DerivedPalette {
    fn style(&self, id: StyleId) -> Style {
        for &(oid, ref s) in &self.overrides {
            if oid == id {
                return *s;
            }
        }
        self.base.style(id)
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

/// A single palette entry for building implementations.
#[derive(Clone, Debug, Default)]
pub struct PaletteStyle {
    fg: Option<crate::cell::Color>,
    bg: Option<crate::cell::Color>,
    bold: bool,
    italic: bool,
    underline: bool,
    dim: bool,
}

impl PaletteStyle {
    pub const fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
            dim: false,
        }
    }
    pub const fn fg(color: crate::cell::Color) -> Self {
        Self {
            fg: Some(color),
            bg: None,
            bold: false,
            italic: false,
            underline: false,
            dim: false,
        }
    }
    pub const fn bg(color: crate::cell::Color) -> Self {
        Self {
            fg: None,
            bg: Some(color),
            bold: false,
            italic: false,
            underline: false,
            dim: false,
        }
    }
    pub const fn colors(fg: crate::cell::Color, bg: crate::cell::Color) -> Self {
        Self {
            fg: Some(fg),
            bg: Some(bg),
            bold: false,
            italic: false,
            underline: false,
            dim: false,
        }
    }
    pub fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
    pub fn italic(mut self) -> Self {
        self.italic = true;
        self
    }
    pub fn dim(mut self) -> Self {
        self.dim = true;
        self
    }
    pub fn to_style(&self) -> Style {
        Style {
            fg: self.fg.unwrap_or(crate::cell::Color::Reset),
            bg: self.bg.unwrap_or(crate::cell::Color::Reset),
            attrs: crate::cell::Attrs {
                bold: self.bold,
                italic: self.italic,
                underline: self.underline,
                dim: self.dim,
            },
        }
    }
}
