//! Sidekick — generic popup container managed by SidekickManager.
//!
//! The caller creates any View, wraps it in Arc<Mutex<>>, and sends
//! CM_SIDEKICK_SHOW. The manager positions and draws it.
//! The caller mutates the shared View directly. CM_SIDEKICK_HIDE removes it.

use std::sync::{Arc, Mutex};

use txv_core::prelude::*;

use crate::tiled_workspace::commands::CM_TW_MAX;

/// Command: show a popup View at given rect.
/// Data: `Box<SidekickShow>`.
pub const CM_SIDEKICK_SHOW: CommandId = CM_TW_MAX + 20;

/// Command: hide the popup.
pub const CM_SIDEKICK_HIDE: CommandId = CM_TW_MAX + 21;

/// Data payload for CM_SIDEKICK_SHOW.
pub struct SidekickShow {
    /// Rect relative to the emitter view.
    pub(crate) rect: Rect,
    pub(crate) view: Arc<Mutex<dyn View>>,
    /// The view that emitted this (for coordinate translation).
    pub(crate) emitter_id: ViewId,
}
