//! Sidekick — completion popup managed by a host component.
//!
//! The sidekick is a puppet View: InputLine populates it and brokers keys to it.
//! A SidekickManager at the desktop level handles placement and drawing.

use txv_core::prelude::*;

use crate::tiled_workspace::commands::CM_TW_MAX;

/// Command: show sidekick at given rect with items.
/// Data: `Box<SidekickShow>`.
pub const CM_SIDEKICK_SHOW: CommandId = CM_TW_MAX + 1;

/// Command: hide the sidekick.
pub const CM_SIDEKICK_HIDE: CommandId = CM_TW_MAX + 2;

/// Command: update sidekick items and selection.
/// Data: `Box<SidekickUpdate>`.
pub const CM_SIDEKICK_UPDATE: CommandId = CM_TW_MAX + 3;

/// Data payload for CM_SIDEKICK_SHOW.
pub struct SidekickShow {
    pub rect: Rect,
    pub items: Vec<String>,
    pub selected: usize,
}

/// Data payload for CM_SIDEKICK_UPDATE.
pub struct SidekickUpdate {
    pub items: Vec<String>,
    pub selected: usize,
}

/// Sidekick view — draws a list of completion candidates.
/// This is a puppet: it does not handle keys directly.
pub struct SidekickView {
    state: ViewState,
    pub(crate) items: Vec<String>,
    selected: usize,
}

impl SidekickView {
    pub fn new() -> Self {
        Self {
            state: ViewState::default(),
            items: Vec::new(),
            selected: 0,
        }
    }

    pub fn set_items(&mut self, items: Vec<String>, selected: usize) {
        self.items = items;
        self.selected = selected.min(self.items.len().saturating_sub(1));
        self.state.mark_dirty();
    }

    pub fn selected_text(&self) -> Option<&str> {
        self.items.get(self.selected).map(|s| s.as_str())
    }

    pub fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1).min(self.items.len() - 1);
            self.state.mark_dirty();
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.state.mark_dirty();
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl View for SidekickView {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let pal = txv_core::palette::palette();
        let normal = pal.style(StyleId::PopupBackground);
        let sel = pal.style(StyleId::PopupSelected);
        for row in 0..h as usize {
            if row >= self.items.len() {
                break;
            }
            let y = row as u16;
            let style = if row == self.selected {
                sel
            } else {
                normal
            };
            self.state.buffer_mut().hline(0, y, w, ' ', style);
            let label = &self.items[row];
            let max = (w as usize).saturating_sub(1);
            let display: String = label.chars().take(max).collect();
            self.state.buffer_mut().print(1, y, &display, style);
        }
    }

    fn handle(&mut self, _event: &Event) -> HandleResult {
        // Puppet — does not handle events directly.
        HandleResult::Ignored
    }
}

impl Default for SidekickView {
    fn default() -> Self {
        Self::new()
    }
}
