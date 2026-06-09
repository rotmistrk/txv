//! ModalKey — View implementation and event handling.

use txv_core::commands::{CM_CANCEL, CM_OK};
use txv_core::palette::palette;
use txv_core::prelude::*;

use super::ModalKey;

impl View for ModalKey {
    delegate_group_state!(group, override { options, draw, handle, set_sink, desired_width });

    fn desired_width(&self) -> u16 {
        if !self.active {
            return 0;
        }
        // prompt + left cap + right cap + child desired widths
        let prompt_w = self.prompt.len() as u16 + 2;
        let child_w: u16 = (0..self.group.child_count())
            .filter_map(|i| self.group.child(i))
            .map(|c| c.desired_width().max(c.bounds().w()))
            .sum();
        prompt_w + child_w
    }

    fn set_sink(&mut self, sink: EventSink) {
        self.group.set_own_sink(sink.clone());
        self.parent_sink = Some(sink);
    }

    fn options(&self) -> ViewOptions {
        ViewOptions::default().with_preprocess().with_modal_cond(self.active)
    }

    fn draw(&mut self) {
        let bounds = self.group.bounds();
        if bounds.w() == 0 || bounds.h() == 0 {
            return;
        }
        let style = palette().style(StyleId::StatusBar);

        if self.active {
            self.draw_active(style);
        } else {
            self.group.buffer_mut().fill(' ', style);
            if !self.idle_label.is_empty() {
                self.group.buffer_mut().print(1, 0, &self.idle_label, style);
            }
        }
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
    fn draw_active(&mut self, bar_style: Style) {
        let modal_style = palette().style(StyleId::StatusBarModal);
        self.group.buffer_mut().fill(' ', modal_style);
        let modal_bg = modal_style.bg();
        let cap_style = Style::new(modal_bg, bar_style.bg());
        self.group.buffer_mut().print(0, 0, "\u{e0b6}", cap_style);
        let prompt_style = modal_style.with_attrs(modal_style.attrs().bold());
        self.group.buffer_mut().print(1, 0, &self.prompt, prompt_style);
        // Blit children between caps
        for i in 0..self.group.child_count() {
            if self.group.is_child_visible(i) {
                self.group.blit_child(i);
            }
        }
        // Right cap AFTER children blit (must not be overwritten)
        let bounds = self.group.bounds();
        let rw = bounds.w().saturating_sub(1);
        let rcap_style = Style::new(modal_bg, bar_style.bg());
        self.group.buffer_mut().print(rw, 0, "\u{e0b4}", rcap_style);
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
        let mut had_terminal = false;
        for ev in events {
            match ev {
                Event::Command { id, data, .. } => {
                    let is_terminal = match self.terminal_command {
                        Some(tc) => id == tc || id == CM_CANCEL,
                        None => id == CM_OK || id == CM_CANCEL,
                    };
                    if is_terminal {
                        had_terminal = true;
                    }
                    // ALL commands go to parent sink
                    self.group.put_command(id, data);
                }
                other => self.group.put_event(other),
            }
        }
        had_terminal
    }

    pub(crate) fn layout_children_modal(&mut self) {
        let prompt_w = self.prompt.len() as u16;
        // +1 for left power cap, +1 for right power cap
        let base_x = prompt_w + 1;
        let total_w = self.group.bounds().w().saturating_sub(base_x + 1);
        let n = self.group.child_count();
        let mut x = base_x;
        for i in 0..n {
            let cw = if i == n - 1 {
                // Last child gets all remaining space
                total_w.saturating_sub(x - base_x)
            } else {
                self.group.child(i).map_or(0, |c| c.bounds().w())
            };
            self.group.set_child_bounds(i, Rect::new(x, 0, cw, 1));
            x += cw;
        }
    }

    fn handle_dormant(&mut self, event: &Event) -> HandleResult {
        if let Event::Key(key) = event {
            if self.trigger_keys.contains(key) {
                self.activate();
                return HandleResult::Consumed;
            }
        }
        if let Event::Command { id, data, .. } = event {
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
