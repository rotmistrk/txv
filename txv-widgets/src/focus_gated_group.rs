//! FocusGatedGroup — a Group that activates/deactivates via commands.
//!
//! When inactive: size is 0, events are ignored, draw is a no-op.
//! When active: renders children, dispatches events to them normally.
//!
//! Activation is command-driven: the associated widget sends
//! `CM_ACTIVATE_GROUP(group_id)` on focus and `CM_DEACTIVATE_GROUP(group_id)`
//! on blur.

use txv_core::buffer::Buffer;
use txv_core::event::Event;
use txv_core::geometry::Rect;
use txv_core::group::GroupState;
use txv_core::prelude::*;

/// Command to activate a FocusGatedGroup by ID.
pub const CM_ACTIVATE_GROUP: CommandId = 160;
/// Command to deactivate a FocusGatedGroup by ID.
pub const CM_DEACTIVATE_GROUP: CommandId = 161;

/// A Group container that is invisible and inert when inactive.
pub struct FocusGatedGroup {
    group: GroupState,
    active: bool,
    group_id: u16,
}

impl FocusGatedGroup {
    pub fn new(group_id: u16) -> Self {
        let group = GroupState::new(ViewOptions {
            preprocess: true,
            focusable: false,
            ..ViewOptions::default()
        });
        Self {
            group,
            active: false,
            group_id,
        }
    }

    pub fn add_child(&mut self, child: Box<dyn View>) {
        self.group.insert(child);
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    fn layout_children(&mut self) {
        let mut x: u16 = 0;
        for i in 0..self.group.child_count() {
            let cw = self.group.child(i).map_or(0, |c| c.bounds().w);
            self.group.set_child_bounds(i, Rect::new(x, 0, cw, 1));
            x += cw;
        }
    }

    fn draw_children(&mut self) {
        let buf_ptr = self.group.buffer_mut() as *mut Buffer;
        for i in 0..self.group.child_count() {
            if let Some(child) = self.group.child_mut(i) {
                if child.bounds().w > 0 {
                    child.draw();
                }
            }
            if let Some(child) = self.group.child(i) {
                let (ox, oy) = self.group.child_origin(i);
                if child.bounds().w > 0 {
                    unsafe { (*buf_ptr).blit(child.buffer(), ox, oy) };
                }
            }
        }
    }
}

impl View for FocusGatedGroup {
    delegate_group_state!(group, override { bounds, draw, handle, set_sink });

    fn set_sink(&mut self, sink: EventSink) {
        self.group.set_own_sink(sink);
    }

    fn bounds(&self) -> Rect {
        let b = self.group.bounds();
        if self.active {
            b
        } else {
            Rect {
                x: b.x,
                y: b.y,
                w: 0,
                h: 1,
            }
        }
    }

    fn draw(&mut self) {
        if !self.active {
            self.group.mark_redrawn();
            return;
        }
        let bounds = self.group.bounds();
        if bounds.w == 0 || bounds.h == 0 {
            self.group.mark_redrawn();
            return;
        }
        let style = txv_core::palette::palette().style(StyleId::StatusBar);
        self.group.buffer_mut().fill(' ', style);
        self.layout_children();
        self.draw_children();
        self.group.mark_redrawn();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // Always listen for activate/deactivate commands
        if let Event::Command { id, data, .. } = event {
            if *id == CM_ACTIVATE_GROUP {
                if let Some(gid) = data.as_ref().and_then(|d| d.downcast_ref::<u16>()) {
                    if *gid == self.group_id {
                        self.active = true;
                        self.group.mark_dirty();
                        return HandleResult::Consumed;
                    }
                }
            }
            if *id == CM_DEACTIVATE_GROUP {
                if let Some(gid) = data.as_ref().and_then(|d| d.downcast_ref::<u16>()) {
                    if *gid == self.group_id {
                        self.active = false;
                        self.group.mark_dirty();
                        return HandleResult::Consumed;
                    }
                }
            }
        }
        if !self.active {
            return HandleResult::Ignored;
        }
        self.group.dispatch(event)
    }
}
