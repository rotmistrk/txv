//! ModalKey — a Group that activates on a trigger, shows prompt + children.
//!
//! Dormant: shows idle label, children invisible/inactive.
//! Active: shows prompt + children, goes modal, deactivates on CM_OK/CM_CANCEL.

use std::time::Instant;

use txv_core::prelude::*;

/// A Group-based modal key handler for the status bar.
///
/// When dormant, shows `idle_label` and intercepts only its trigger key.
/// When active, shows `prompt` followed by children, dispatches events to them.
/// Deactivates when a child emits CM_OK or CM_CANCEL, or on timeout.
pub struct ModalKey {
    group: GroupState,
    idle_label: String,
    prompt: String,
    trigger_keys: Vec<KeyEvent>,
    trigger_command: Option<CommandId>,
    active: bool,
    timeout_secs: Option<u16>,
    cancel_on_miss: bool,
    activated_at: Option<Instant>,
    child_sink: EventSink,
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

    fn propagate_child_sink(&mut self, idx: usize) {
        if let Some(child) = self.group.child_mut(idx) {
            child.set_sink(self.child_sink.clone());
        }
    }

    fn activate(&mut self) {
        self.active = true;
        self.activated_at = Some(Instant::now());
        self.update_bounds();
    }

    fn deactivate(&mut self) {
        self.active = false;
        self.activated_at = None;
        self.update_bounds();
    }

    fn update_bounds(&mut self) {
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

    fn dormant_width(&self) -> u16 {
        if self.idle_label.is_empty() {
            0
        } else {
            self.idle_label.len() as u16 + 2
        }
    }

    fn active_width(&self) -> u16 {
        let prompt_w = self.prompt.len() as u16;
        let children_w: u16 = (0..self.group.child_count())
            .map(|i| self.group.child(i).map_or(0, |c| c.bounds().w))
            .sum();
        prompt_w + children_w + 2
    }

    fn check_timeout(&mut self) {
        let Some(secs) = self.timeout_secs else {
            return;
        };
        let Some(at) = self.activated_at else {
            return;
        };
        if at.elapsed().as_secs() >= u64::from(secs) {
            self.deactivate();
        }
    }

    fn drain_child_commands(&mut self) -> bool {
        let events = self.child_sink.drain();
        let mut should_deactivate = false;
        for ev in events {
            if let Event::Command { id, .. } = &ev {
                if *id == CM_OK || *id == CM_CANCEL {
                    should_deactivate = true;
                }
            }
            // Forward to parent sink
            self.group.put_event(ev);
        }
        should_deactivate
    }

    fn layout_children(&mut self) {
        let prompt_w = self.prompt.len() as u16;
        let y = self.group.bounds().y;
        let base_x = self.group.bounds().x + prompt_w;
        let mut x = base_x;
        for i in 0..self.group.child_count() {
            let cw = self.group.child(i).map_or(0, |c| c.bounds().w);
            self.group.set_child_bounds(i, Rect::new(x, y, cw, 1));
            x += cw;
        }
    }
}

impl View for ModalKey {
    delegate_group_state!(group, override { options, draw, handle, set_sink });

    fn set_sink(&mut self, sink: EventSink) {
        // Set parent sink on the group itself, but keep children on child_sink
        self.group.set_own_sink(sink);
    }

    fn options(&self) -> ViewOptions {
        ViewOptions {
            preprocess: true,
            focusable: false,
            modal: self.active,
            ..ViewOptions::default()
        }
    }

    fn draw(&mut self) {
        let bounds = self.group.bounds();
        if bounds.w == 0 || bounds.h == 0 {
            self.group.mark_redrawn();
            return;
        }
        let style = Style {
            attrs: Attrs {
                reverse: true,
                ..Attrs::default()
            },
            ..Style::default()
        };
        self.group.buffer_mut().fill(' ', style);

        if self.active {
            self.layout_children();
            self.group.buffer_mut().print(0, 0, &self.prompt, style);
            self.draw_children(bounds, style);
        } else if !self.idle_label.is_empty() {
            self.group.buffer_mut().print(1, 0, &self.idle_label, style);
        }
        self.group.mark_redrawn();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Tick = event {
            if self.active {
                self.check_timeout();
            }
            return HandleResult::Ignored;
        }

        if !self.active {
            return self.handle_dormant(event);
        }
        self.handle_active(event)
    }
}

impl ModalKey {
    fn draw_children(&mut self, bounds: Rect, _style: Style) {
        let buf_ptr = self.group.buffer_mut() as *mut Buffer;
        for i in 0..self.group.child_count() {
            if let Some(child) = self.group.child_mut(i) {
                if child.bounds().w > 0 {
                    child.draw();
                }
            }
            if let Some(child) = self.group.child(i) {
                let cb = child.bounds();
                if cb.w > 0 {
                    let dx = cb.x.saturating_sub(bounds.x);
                    let dy = cb.y.saturating_sub(bounds.y);
                    unsafe { (*buf_ptr).blit(child.buffer(), dx, dy) };
                }
            }
        }
    }

    fn handle_dormant(&mut self, event: &Event) -> HandleResult {
        if let Event::Key(key) = event {
            if self.trigger_keys.contains(key) {
                self.activate();
                return HandleResult::Consumed;
            }
        }
        if let Event::Command { id, data } = event {
            if Some(*id) == self.trigger_command {
                // Allow command data to override prompt
                if let Some(text) = data.as_ref().and_then(|d| d.downcast_ref::<String>()) {
                    self.prompt = text.clone();
                }
                self.activate();
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }

    fn handle_active(&mut self, event: &Event) -> HandleResult {
        // Dispatch to children
        self.group.dispatch(event);

        // Check if children emitted deactivation commands
        if self.drain_child_commands() {
            self.deactivate();
            return HandleResult::Consumed;
        }

        // cancel_on_miss: if key event and no child consumed it
        if self.cancel_on_miss {
            if let Event::Key(_) = event {
                self.deactivate();
            }
        }

        HandleResult::Consumed
    }
}
