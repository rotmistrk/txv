//! FocusGatedGroup — a status bar item that shows/hides based on focus commands.
//!
//! Dormant: width=0, events ignored, draw is no-op.
//! Active: renders children (key labels), dispatches events to them.
//!
//! Follows the same pattern as ModalKey: manages own bounds via set_bounds.

use txv_core::event::{Event, KeyCode, KeyEvent, KeyMod};
use txv_core::geometry::Rect;
use txv_core::group::GroupState;
use txv_core::palette::palette;
use txv_core::prelude::*;

/// Command to activate a FocusGatedGroup by ID.
pub const CM_ACTIVATE_GROUP: CommandId = 160;
/// Command to deactivate a FocusGatedGroup by ID.
pub const CM_DEACTIVATE_GROUP: CommandId = 161;

pub struct FocusGatedGroup {
    group: GroupState,
    active: bool,
    group_id: u16,
    natural_width: u16,
}

impl FocusGatedGroup {
    pub fn new(group_id: u16) -> Self {
        let mut group = GroupState::new(ViewOptions::default().with_preprocess());
        // Start with zero width (dormant).
        group.set_bounds(Rect::new(0, 0, 0, 1));
        Self {
            group,
            active: false,
            group_id,
            natural_width: 0,
        }
    }

    pub fn add_child(&mut self, child: Box<dyn View>) {
        self.natural_width += child.bounds().w();
        self.group.insert(child);
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    fn activate(&mut self) {
        self.active = true;
        self.sync_width();
        self.group.mark_dirty();
    }

    fn deactivate(&mut self) {
        self.active = false;
        // Cancel any active modal children (send Esc)
        let esc = Event::Key(KeyEvent::new(KeyCode::Esc, KeyMod::default()));
        self.group.dispatch(&esc);
        let b = self.group.bounds();
        self.group.set_bounds(Rect::new(b.x(), b.y(), 0, 1));
        self.group.mark_dirty();
    }

    fn layout_children(&mut self) {
        let mut x: u16 = 0;
        for i in 0..self.group.child_count() {
            let cw = self.group.child(i).map_or(0, |c| c.bounds().w());
            self.group.set_child_bounds(i, Rect::new(x, 0, cw, 1));
            x += cw;
        }
    }
}

impl View for FocusGatedGroup {
    delegate_group_state!(group, override { draw, handle, set_sink, set_bounds });

    fn set_bounds(&mut self, r: Rect) {
        self.group.set_bounds(r);
        if self.active {
            self.layout_children();
        }
    }

    fn set_sink(&mut self, sink: EventSink) {
        self.group.set_sink(sink);
    }

    fn draw(&mut self) {
        if !self.active {
            return;
        }
        let bounds = self.group.bounds();
        if bounds.w() == 0 || bounds.h() == 0 {
            return;
        }
        let style = palette().style(StyleId::StatusBar);
        self.group.buffer_mut().fill(' ', style);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Command { id, data, .. } = event {
            if let Some(r) = self.handle_gate_command(*id, data) {
                return r;
            }
        }
        if !self.active {
            return HandleResult::Ignored;
        }
        let result = self.group.dispatch(event);
        // Update own width from children (ModalKey may have expanded/collapsed)
        self.sync_width();
        result
    }
}

impl FocusGatedGroup {
    fn sync_width(&mut self) {
        let mut w: u16 = 0;
        for i in 0..self.group.child_count() {
            w += self.group.child(i).map_or(0, |c| c.bounds().w());
        }
        let b = self.group.bounds();
        if b.w() != w {
            self.group.set_bounds(Rect::new(b.x(), b.y(), w, 1));
            self.layout_children();
            self.group.mark_dirty();
        }
    }
    fn handle_gate_command(
        &mut self,
        id: CommandId,
        data: &Option<Box<dyn std::any::Any + Send>>,
    ) -> Option<HandleResult> {
        if id == CM_ACTIVATE_GROUP {
            let gid = data.as_ref().and_then(|d| d.downcast_ref::<u16>())?;
            if *gid == self.group_id {
                self.activate();
                return Some(HandleResult::Consumed);
            }
        }
        if id == CM_DEACTIVATE_GROUP {
            let gid = data.as_ref().and_then(|d| d.downcast_ref::<u16>())?;
            if *gid == self.group_id || *gid == u16::MAX {
                self.deactivate();
                if *gid == u16::MAX {
                    return None; // Wildcard: don't consume, let others see it
                }
                return Some(HandleResult::Consumed);
            }
        }
        None
    }
}
