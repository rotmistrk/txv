//! ModalKey — View implementation and event handling.

use txv_core::cell::Attrs;
use txv_core::prelude::*;

use super::ModalKey;

impl View for ModalKey {
    delegate_group_state!(group, override { options, draw, handle, set_sink });

    fn set_sink(&mut self, sink: EventSink) {
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
        if self.active {
            self.update_bounds();
        }
        let bounds = self.group.bounds();
        if bounds.w == 0 || bounds.h == 0 {
            self.group.mark_redrawn();
            return;
        }
        let style = txv_core::palette::palette().style(StyleId::StatusBar);

        if self.active {
            // Active modal: distinct background from status bar
            let modal_style = txv_core::palette::palette().style(StyleId::StatusBarModal);
            self.group.buffer_mut().fill(' ', modal_style);
            // Left power cap: modal bg fg on status_bar bg
            let modal_bg = modal_style.bg;
            let cap_style = Style {
                fg: modal_bg,
                bg: style.bg,
                attrs: Attrs::default(),
            };
            self.group.buffer_mut().print(0, 0, "\u{e0b6}", cap_style);
            // Prompt in bold
            let prompt_style = Style {
                attrs: Attrs {
                    bold: true,
                    ..modal_style.attrs
                },
                ..modal_style
            };
            self.group.buffer_mut().print(1, 0, &self.prompt, prompt_style);
            self.layout_children_modal();
            self.draw_children(bounds);
            // Right power cap: modal bg fg on status_bar bg
            let rw = bounds.w.saturating_sub(1);
            let rcap_style = Style {
                fg: modal_bg,
                bg: style.bg,
                attrs: Attrs::default(),
            };
            self.group.buffer_mut().print(rw, 0, "\u{e0b4}", rcap_style);
        } else {
            self.group.buffer_mut().fill(' ', style);
            if !self.idle_label.is_empty() {
                self.group.buffer_mut().print(1, 0, &self.idle_label, style);
            }
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
        let mut had_terminal = false;
        for ev in events {
            if let Event::Command { id, .. } = &ev {
                if !Self::is_passthrough_command(*id) {
                    had_terminal = true;
                }
            }
            self.group.put_event(ev);
        }
        had_terminal
    }

    /// Commands that should pass through without deactivating the modal.
    fn is_passthrough_command(id: CommandId) -> bool {
        use crate::sidekick::{CM_SIDEKICK_HIDE, CM_SIDEKICK_SHOW};
        matches!(id, CM_SIDEKICK_SHOW | CM_SIDEKICK_HIDE)
    }

    fn layout_children_modal(&mut self) {
        let prompt_w = self.prompt.len() as u16;
        let y = self.group.bounds().y;
        // +1 for left power cap
        let base_x = self.group.bounds().x + prompt_w + 1;
        let mut x = base_x;
        for i in 0..self.group.child_count() {
            let cw = self.group.child(i).map_or(0, |c| c.bounds().w);
            self.group.set_child_bounds(i, Rect::new(x, y, cw, 1));
            x += cw;
        }
    }

    fn draw_children(&mut self, bounds: Rect) {
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
                if let Some(text) = data.as_ref().and_then(|d| d.downcast_ref::<String>()) {
                    self.prompt = text.clone();
                }
                self.activate();
                return HandleResult::Consumed;
            }
            if Some(*id) == self.prefill_command {
                self.activate();
                self.group.dispatch(event);
                self.child_sink.drain();
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }

    fn handle_active(&mut self, event: &Event) -> HandleResult {
        let result = self.group.dispatch(event);

        if self.drain_child_commands() {
            self.deactivate();
            return HandleResult::Consumed;
        }

        // Update bounds after children may have resized
        self.update_bounds();

        if self.cancel_on_miss && result == HandleResult::Ignored {
            if let Event::Key(_) = event {
                self.deactivate();
            }
        }

        // Only consume key events — let commands pass through to postprocess.
        match event {
            Event::Key(_) => HandleResult::Consumed,
            _ => result,
        }
    }
}
