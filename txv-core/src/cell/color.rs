//! Color — terminal color representation.

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Color {
    #[default]
    Reset,
    /// Transparent — blit skips cells where both fg and bg are Transparent.
    Transparent,
    Ansi(u8),
    Palette(u8),
    Rgb(u8, u8, u8),
}
