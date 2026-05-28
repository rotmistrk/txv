//! Palette trait definitions — sub-palettes as traits.

use crate::cell::Style;

/// Top-level palette: provides access to sub-palettes.
pub trait Palette: Send + Sync {
    fn base(&self) -> &dyn Base;
    fn interactive(&self) -> &dyn Interactive;
    fn chrome(&self) -> &dyn Chrome;
    fn popup(&self) -> &dyn Popup;
    fn state(&self) -> &dyn State;
}

/// Base text styles.
pub trait Base {
    fn text(&self) -> Style;
    fn dim(&self) -> Style;
    fn bright(&self) -> Style;
    fn border(&self) -> Style;
    fn separator(&self) -> Style;
    fn tree_dir(&self) -> Style;
}

/// Interactive element styles.
pub trait Interactive {
    fn cursor_focused(&self) -> Style;
    fn cursor_unfocused(&self) -> Style;
    fn input_cursor(&self) -> Style;
    fn edit_overlay(&self) -> Style;
    fn edit_selection(&self) -> Style;
    fn search_match(&self) -> Style;
    fn visual_selection(&self) -> Style;
    fn disabled(&self) -> Style;
}

/// Chrome (UI frame) styles.
pub trait Chrome {
    fn bar(&self) -> Style;
    fn tab_focused(&self) -> Style;
    fn tab_focused_arrow(&self) -> Style;
    fn tab_focused_badge(&self) -> Style;
    fn tab_active(&self) -> Style;
    fn tab_active_arrow(&self) -> Style;
    fn tab_active_badge(&self) -> Style;
    fn tab_inactive(&self, distance: usize) -> Style;
    fn status_bar(&self) -> Style;
    fn scrollbar_track(&self) -> Style;
    fn scrollbar_thumb(&self) -> Style;
}

/// Popup/dialog styles.
pub trait Popup {
    fn background(&self) -> Style;
    fn border(&self) -> Style;
    fn selected(&self) -> Style;
    fn table_header(&self) -> Style;
}

/// State indication styles.
pub trait State {
    fn error(&self) -> Style;
    fn warning(&self) -> Style;
    fn info(&self) -> Style;
    fn success(&self) -> Style;
    fn hint(&self) -> Style;
}
