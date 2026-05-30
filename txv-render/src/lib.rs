//! # txv-render
//!
//! Terminal backend for the txv TUI framework.
//! Implements `txv_core::Backend` via crossterm, provides a VTE terminal
//! emulator (`TermBuf`), color mode detection/downgrade, and text utilities.

pub mod backend;
mod backend_flush;
pub mod color;
pub mod diff;
mod event_translate;
mod style_emit;
pub mod termbuf;
pub mod text;

pub use backend::CrosstermBackend;
pub use color::{detect_color_mode, downgrade, ColorMode};
pub use diff::diff_cells;
pub use termbuf::{TCell, TermBuf};
pub use text::{byte_to_col, col_to_byte, display_width, truncate, wrap};
