//! # txv-edit
//!
//! Reusable text editor with vi-style keybindings for TUI applications.
//!
//! ## Architecture
//!
//! - `Editor` — buffer (PieceTable), cursor, mode, registers, search, vi keymap
//! - `EditorAction` — result enum emitted by editor after executing commands
//! - `EditorCore` — lower-level buffer + cursor (may be unified with Editor later)
//! - `KeymapHandler` — trait for pluggable keybinding schemes (vi, emacs, etc.)
//! - `EditorHost` — trait for app-specific integration (clipboard, :commands, completion)

pub mod buffer;
pub mod editor;
pub mod editor_core;
pub mod editor_host;
pub mod keymap;
pub mod settings;
pub mod shared_register;
