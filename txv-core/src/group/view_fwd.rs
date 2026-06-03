//! Forwarding methods from GroupState to its inner ViewState.

use crate::view::{EventSink, ViewOptions};

use super::GroupState;

impl GroupState {
    pub fn bounds(&self) -> crate::geometry::Rect {
        self.view.bounds()
    }

    pub fn set_bounds(&mut self, r: crate::geometry::Rect) {
        self.view.set_bounds(r);
    }

    pub fn mark_dirty(&mut self) {
        self.view.mark_dirty();
    }

    pub fn mark_redrawn(&mut self) {
        self.view.mark_redrawn();
    }

    pub fn is_dirty(&self) -> bool {
        self.view.is_dirty()
    }

    pub fn is_focused(&self) -> bool {
        self.view.is_focused()
    }

    pub fn set_focused(&mut self, f: bool) {
        self.view.set_focused(f);
    }

    pub fn buffer(&self) -> &crate::buffer::Buffer {
        self.view.buffer()
    }

    pub fn buffer_mut(&mut self) -> &mut crate::buffer::Buffer {
        self.view.buffer_mut()
    }

    pub fn options(&self) -> ViewOptions {
        self.view.options()
    }

    pub fn sink(&self) -> Option<&EventSink> {
        self.view.sink()
    }

    /// Set the sink on this group only, without propagating to children.
    pub fn set_own_sink(&mut self, sink: EventSink) {
        self.view.set_sink(sink);
    }

    pub fn title(&self) -> &str {
        self.view.title()
    }

    pub fn set_title(&mut self, t: impl Into<String>) {
        self.view.set_title(t);
    }

    pub fn put_event(&self, event: crate::event::Event) {
        self.view.put_event(event);
    }

    pub fn put_command(&self, id: crate::event::CommandId, data: Option<Box<dyn std::any::Any + Send>>) {
        self.view.put_command(id, data);
    }

    /// Query the focused child's cursor request and translate to group-relative coords.
    /// Preprocess children that report a cursor take priority (they capture input when active).
    pub fn cursor(&self) -> Option<crate::cursor::CursorRequest> {
        // Check preprocess children first — if they report a cursor, they're active.
        for (i, child) in self.children.iter().enumerate() {
            if child.options().preprocess {
                if let Some(mut req) = child.cursor() {
                    let (ox, oy) = self.origins.get(i).copied().unwrap_or((0, 0));
                    req.x = req.x.saturating_add(ox);
                    req.y = req.y.saturating_add(oy);
                    return Some(req);
                }
            }
        }
        let child = self.focused_child()?;
        let mut req = child.cursor()?;
        let (ox, oy) = self.origins.get(self.focused).copied().unwrap_or((0, 0));
        req.x = req.x.saturating_add(ox);
        req.y = req.y.saturating_add(oy);
        Some(req)
    }
}
