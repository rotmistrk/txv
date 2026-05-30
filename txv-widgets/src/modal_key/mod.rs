//! ModalKey — a Group that activates on a trigger, shows prompt + children.
//!
//! Dormant: shows idle label, children invisible/inactive.
//! Active: shows prompt + children, goes modal, deactivates on child commands.

mod view_impl;

use std::time::Instant;

use txv_core::prelude::*;

/// A Group-based modal key handler for the status bar.
///
/// When dormant, shows `idle_label` and intercepts only its trigger key.
/// When active, shows `prompt` followed by children, dispatches events to them.
/// Deactivates when a child emits any command, or on timeout.
pub struct ModalKey {
    pub(crate) group: GroupState,
    pub(crate) idle_label: String,
    pub(crate) prompt: String,
    pub(crate) trigger_keys: Vec<KeyEvent>,
    pub(crate) trigger_command: Option<CommandId>,
    pub(crate) prefill_command: Option<CommandId>,
    pub(crate) active: bool,
    pub(crate) timeout_secs: Option<u16>,
    pub(crate) cancel_on_miss: bool,
    pub(crate) activated_at: Option<Instant>,
    pub(crate) child_sink: EventSink,
}

impl ModalKey {
    pub fn new(idle_label: impl Into<String>, prompt: impl Into<String>) -> Self {
        let mut group = GroupState::new(ViewOptions {
            preprocess: true,
            focusable: false,
            ..ViewOptions::default()
        });
        let idle = idle_label.into();
        let w = if idle.is_empty() {
            0
        } else {
            idle.len() as u16 + 2
        };
        group.set_bounds(Rect::new(0, 0, w, 1));
        Self {
            group,
            idle_label: idle,
            prompt: prompt.into(),
            trigger_keys: Vec::new(),
            trigger_command: None,
            prefill_command: None,
            active: false,
            timeout_secs: None,
            cancel_on_miss: false,
            activated_at: None,
            child_sink: EventSink::new(),
        }
    }

    pub fn trigger_key(mut self, key: KeyEvent) -> Self {
        self.trigger_keys.push(key);
        self
    }

    pub fn trigger_command(mut self, cmd: CommandId) -> Self {
        self.trigger_command = Some(cmd);
        self
    }

    pub fn prefill_command(mut self, cmd: CommandId) -> Self {
        self.prefill_command = Some(cmd);
        self
    }

    pub fn timeout(mut self, secs: u16) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    pub fn cancel_on_miss(mut self) -> Self {
        self.cancel_on_miss = true;
        self
    }

    pub fn add_child(mut self, child: Box<dyn View>) -> Self {
        self.group.insert(child);
        let idx = self.group.child_count() - 1;
        self.propagate_child_sink(idx);
        self
    }

    pub(crate) fn propagate_child_sink(&mut self, idx: usize) {
        if let Some(child) = self.group.child_mut(idx) {
            child.set_sink(self.child_sink.clone());
        }
    }

    pub(crate) fn activate(&mut self) {
        self.active = true;
        self.activated_at = Some(Instant::now());
        self.propagate_modal_palette();
        for i in 0..self.group.child_count() {
            if let Some(child) = self.group.child_mut(i) {
                child.select();
            }
        }
        // Request minimum active width so parent layout can drop lower-priority items
        let b = self.group.bounds();
        let min_active = self.active_min_width();
        if b.w < min_active {
            self.group.set_bounds(Rect::new(b.x, b.y, min_active, b.h));
        }
        self.group.mark_dirty();
    }

    pub(crate) fn deactivate(&mut self) {
        self.active = false;
        self.activated_at = None;
        for i in 0..self.group.child_count() {
            if let Some(child) = self.group.child_mut(i) {
                child.unselect();
            }
        }
        self.propagate_default_palette();
        self.update_bounds();
    }

    pub(crate) fn propagate_modal_palette(&mut self) {
        use std::sync::Arc;
        use txv_core::palette::{DerivedPalette, Palette, StyleId};
        let base = txv_core::palette::palette();
        let modal_style = base.style(StyleId::StatusBarModal);
        let derived: Arc<dyn Palette> = Arc::new(
            DerivedPalette::new(base)
                .with_override(StyleId::Text, modal_style)
                .with_override(StyleId::StatusBar, modal_style),
        );
        for i in 0..self.group.child_count() {
            if let Some(child) = self.group.child_mut(i) {
                child.set_palette(derived.clone());
            }
        }
    }

    pub(crate) fn propagate_default_palette(&mut self) {
        let pal = txv_core::palette::palette();
        for i in 0..self.group.child_count() {
            if let Some(child) = self.group.child_mut(i) {
                child.set_palette(pal.clone());
            }
        }
    }

    pub(crate) fn update_bounds(&mut self) {
        let w = if self.active {
            self.active_width()
        } else {
            self.dormant_width()
        };
        let b = self.group.bounds();
        if b.w != w {
            self.group.set_bounds(Rect::new(b.x, b.y, w, b.h));
        }
        self.group.mark_dirty();
    }

    pub(crate) fn dormant_width(&self) -> u16 {
        if self.idle_label.is_empty() {
            0
        } else {
            self.idle_label.len() as u16 + 2
        }
    }

    pub(crate) fn active_width(&self) -> u16 {
        let prompt_w = self.prompt.len() as u16;
        let children_w: u16 = (0..self.group.child_count())
            .map(|i| self.group.child(i).map_or(0, |c| c.bounds().w))
            .sum();
        // +2 for power caps (left + right)
        prompt_w + children_w + 3
    }

    /// Minimum width when active (prompt + caps + reasonable input space).
    fn active_min_width(&self) -> u16 {
        let prompt_w = self.prompt.len() as u16;
        // prompt + 2 caps + at least 20 chars for input
        prompt_w + 2 + 20
    }
}
