//! Program — the correct way to build a TXV application.
//!
//! Program handles the event loop, three-phase dispatch, draw cycle,
//! resize, and quit. The application only provides:
//! - A desktop view (the main content)
//! - A status bar view (preprocess, key→command translation)
//! - A command handler (what to do when commands arrive)

mod command_context;
mod program_impl;

pub use command_context::CommandContext;
pub use program_impl::Program;
