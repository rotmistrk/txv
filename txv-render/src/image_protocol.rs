//! Terminal image protocol detection and cell size query.

use std::env;

/// Supported terminal image protocols.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageProtocol {
    /// No image support detected.
    None,
    /// Kitty graphics protocol (direct RGBA upload).
    Kitty,
    /// iTerm2 inline image protocol (base64 PNG in OSC 1337).
    Iterm2,
}

/// Cell dimensions in pixels (for mapping cell rects to pixel regions).
#[derive(Clone, Copy, Debug)]
pub struct CellPixelSize {
    width: u16,
    height: u16,
}

impl CellPixelSize {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }
}

impl Default for CellPixelSize {
    fn default() -> Self {
        Self { width: 8, height: 16 }
    }
}

/// Detect the terminal's image protocol from environment variables.
pub fn detect_image_protocol() -> ImageProtocol {
    if let Ok(term_program) = env::var("TERM_PROGRAM") {
        let lc = term_program.to_lowercase();
        if lc.contains("kitty") {
            return ImageProtocol::Kitty;
        }
        if lc.contains("iterm") || lc.contains("wezterm") {
            return ImageProtocol::Iterm2;
        }
    }
    if let Ok(term) = env::var("TERM") {
        if term.contains("kitty") {
            return ImageProtocol::Kitty;
        }
    }
    // Ghostty supports Kitty protocol
    if env::var("GHOSTTY_RESOURCES_DIR").is_ok() {
        return ImageProtocol::Kitty;
    }
    ImageProtocol::None
}
