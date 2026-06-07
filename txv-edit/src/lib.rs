//! # txv-edit
//!
//! Reusable text editor with vi-style keybindings for TUI applications.
//!
//! ## Architecture
//!
//! - `EditorCore` — buffer (PieceTable), cursor, selections, undo/redo, motions, text operations
//! - `KeymapHandler` — trait for pluggable keybinding schemes (vi, emacs, etc.)
//! - `vi` — default KeymapHandler implementing modal vi editing
//! - `EditorHost` — trait for app-specific integration (clipboard, :commands, completion)
//! - `highlight` — syntax highlighting with extension/shebang/fragment autodetection

pub mod buffer;
pub mod editor_core;
pub mod editor_host;
pub mod keymap;
