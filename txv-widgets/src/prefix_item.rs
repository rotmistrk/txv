//! PrefixItem — two-key sequence handler for StatusBar.
//!
//! On prefix key: goes exclusive, shows available bindings.
//! On second key: emits command, releases exclusive.
//! On Esc/unknown: cancels, releases exclusive.

use txv_core::event::{Event, KeyCode, KeyEvent};
use txv_core::status::{ActiveItem, Gravity, VisibleItem};
use txv_core::view::{EventSink, HandleResult};

use crate::prefix_binding::PrefixBinding;

/// Two-key prefix item for StatusBar.
///
/// Idle: invisible (or shows compact label like "C-w").
/// Active: exclusive, shows all bindings.
pub struct PrefixItem {
    prefix_key: KeyEvent,
    bindings: Vec<PrefixBinding>,
    active: bool,
    idle_label: String,
    active_label: String,
}

impl PrefixItem {
    pub fn new(prefix_key: KeyEvent, idle_label: impl Into<String>) -> Self {
        Self {
            prefix_key,
            bindings: Vec::new(),
            active: false,
            idle_label: idle_label.into(),
            active_label: String::new(),
        }
    }

    pub fn bind(mut self, key: char, command: txv_core::event::CommandId, label: &'static str) -> Self {
        self.bindings.push(PrefixBinding { key, command, label });
        self.rebuild_active_label();
        self
    }

    fn rebuild_active_label(&mut self) {
        let parts: Vec<String> = self.bindings.iter().map(|b| format!("{}:{}", b.key, b.label)).collect();
        self.active_label = format!("C-w: {}", parts.join(" "));
    }

    fn dispatch(&self, ch: char, sink: &EventSink) -> bool {
        for b in &self.bindings {
            if b.key == ch {
                sink.push_command(b.command, None);
                return true;
            }
        }
        false
    }
}

impl ActiveItem for PrefixItem {
    fn handle(&mut self, event: &Event, sink: &EventSink) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };

        if !self.active {
            if *key == self.prefix_key {
                self.active = true;
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
                // Ctrl-W Ctrl-W = cycle (treat ctrl+w as 'w')
                let effective = if key.modifiers().ctrl() && ch == 'w' {
                    'w'
                } else {
                    ch
                };
                self.dispatch(effective, sink);
                self.active = false;
            }
            _ => {
                self.active = false;
            }
        }
        HandleResult::Consumed
    }

    fn is_exclusive(&self) -> bool {
        self.active
    }
}

impl VisibleItem for PrefixItem {
    fn label(&self) -> &str {
        if self.active {
            &self.active_label
        } else {
            &self.idle_label
        }
    }

    fn gravity(&self) -> Gravity {
        Gravity::Left
    }
}
