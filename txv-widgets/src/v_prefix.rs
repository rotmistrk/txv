//! PrefixView — two-key sequence handler as a View.
//!
//! Idle: shows compact label. Active: goes modal, shows bindings, dispatches second key.

use std::sync::Arc;

use txv_core::prelude::*;

use crate::prefix_binding::PrefixBinding;
use crate::resize_helpers::resize_width_to;

/// Two-key prefix View for status bar.
pub struct PrefixView {
    state: ViewState,
    palette: Option<Arc<dyn Palette>>,
    prefix_key: KeyEvent,
    bindings: Vec<PrefixBinding>,
    active: bool,
    idle_label: String,
    active_label: String,
}

impl PrefixView {
    pub fn new(prefix_key: KeyEvent, idle_label: impl Into<String>) -> Self {
        let idle_label = idle_label.into();
        let w = idle_label.len() as u16 + 2;
        let mut state = ViewState::new(ViewOptions::default().with_preprocess());
        state.set_bounds(Rect::new(0, 0, w, 1));
        Self {
            state,
            palette: None,
            prefix_key,
            bindings: Vec::new(),
            active: false,
            idle_label,
            active_label: String::new(),
        }
    }

    delegate_palette!(palette);

    pub fn bind(mut self, key: char, command: CommandId, label: &'static str) -> Self {
        self.bindings.push(PrefixBinding { key, command, label });
        self.rebuild_active_label();
        self
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    fn rebuild_active_label(&mut self) {
        let parts: Vec<String> = self.bindings.iter().map(|b| format!("{}:{}", b.key, b.label)).collect();
        self.active_label = format!("{}: {}", self.idle_label, parts.join(" "));
        // Resize buffer to fit active label when active
    }

    fn dispatch_key(&self, ch: char) -> bool {
        for b in &self.bindings {
            if b.key == ch {
                self.state.put_command(b.command, None);
                return true;
            }
        }
        false
    }

    fn resize_for_state(&mut self) {
        let label = if self.active {
            &self.active_label
        } else {
            &self.idle_label
        };
        let w = label.len() as u16 + 2;
        resize_width_to(&mut self.state, w);
    }
}

impl View for PrefixView {
    delegate_view_state!(state, override { options });

    fn options(&self) -> ViewOptions {
        ViewOptions::default().with_preprocess().with_modal_cond(self.active)
    }

    fn draw(&mut self) {
        let label = if self.active {
            &self.active_label
        } else {
            &self.idle_label
        };
        let style = self.resolve_style(StyleId::StatusBar);
        let buf = self.state.buffer_mut();
        buf.fill(' ', style);
        if !label.is_empty() {
            buf.print(1, 0, label, style);
        }
        self.state.mark_redrawn();
    }

    fn set_palette(&mut self, palette: Arc<dyn Palette>) {
        self.palette = Some(palette);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };

        if !self.active {
            if *key == self.prefix_key {
                self.active = true;
                self.resize_for_state();
                self.state.mark_dirty();
                return HandleResult::Consumed;
            }
            return HandleResult::Ignored;
        }

        // Active — waiting for second key
        match key.code() {
            KeyCode::Esc => {
                self.active = false;
            }
            KeyCode::Char(ch) if !key.modifiers().alt() => {
                self.dispatch_key(ch);
                self.active = false;
            }
            _ => {
                self.active = false;
            }
        }
        self.resize_for_state();
        self.state.mark_dirty();
        HandleResult::Consumed
    }
}
