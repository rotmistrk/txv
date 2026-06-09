//! Sidekick — generic popup container managed by SidekickManager.
//!
//! The caller creates a View, sends CM_SIDEKICK_SHOW with ownership.
//! SidekickManager positions and hosts it. CM_SIDEKICK_HIDE removes it.

use txv_core::prelude::*;

use crate::tiled_workspace::commands::CM_TW_MAX;

/// Command: show a popup View at given rect.
/// Data: `Box<SidekickShow>`.
pub const CM_SIDEKICK_SHOW: CommandId = CM_TW_MAX + 20;

/// Command: hide the popup.
pub const CM_SIDEKICK_HIDE: CommandId = CM_TW_MAX + 21;

/// Command: select next item in sidekick popup.
pub const CM_SIDEKICK_NEXT: CommandId = CM_TW_MAX + 22;

/// Command: select previous item in sidekick popup.
pub const CM_SIDEKICK_PREV: CommandId = CM_TW_MAX + 23;

/// Command: apply (confirm) current selection. Sidekick responds with CM_SIDEKICK_RESULT.
pub const CM_SIDEKICK_APPLY: CommandId = CM_TW_MAX + 24;

/// Command: sidekick sends selected text back. Data: Box<String>.
pub const CM_SIDEKICK_RESULT: CommandId = CM_TW_MAX + 25;

/// Data payload for CM_SIDEKICK_SHOW.
pub struct SidekickRequest {
    /// Rect (width/height for the popup).
    pub(crate) rect: Rect,
    /// The view to display (ownership transferred via Mutex for interior mutability).
    view: std::sync::Mutex<Option<Box<dyn View>>>,
    /// The view that emitted this (for coordinate translation).
    pub(crate) emitter_id: ViewId,
}

impl SidekickRequest {
    pub fn new(view: Box<dyn View>, rect: Rect, emitter_id: ViewId) -> Self {
        Self {
            rect,
            view: std::sync::Mutex::new(Some(view)),
            emitter_id,
        }
    }

    /// Take the view out.
    pub fn take_view(&self) -> Option<Box<dyn View>> {
        self.view.lock().ok().and_then(|mut v| v.take())
    }
}
