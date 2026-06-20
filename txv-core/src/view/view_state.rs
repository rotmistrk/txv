//! ViewState — common state embedded in every View.

use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::buffer::Buffer;
use crate::event::{CommandId, Event};
use crate::geometry::Rect;

use super::{EventSink, ViewId, ViewOptions};

static NEXT_VIEW_ID: AtomicU64 = AtomicU64::new(1);

fn next_view_id() -> ViewId {
    NEXT_VIEW_ID.fetch_add(1, Ordering::Relaxed)
}

/// Common view state — embed in every view.
pub struct ViewState {
    id: ViewId,
    bounds: Rect,
    pub(crate) options: ViewOptions,
    dirty: bool,
    focused: bool,
    pub(crate) title: String,
    /// The view's drawing buffer. Sized to bounds. Draw into this in draw().
    buf: Buffer,
    sink: Option<EventSink>,
}

impl ViewState {
    pub fn new(options: ViewOptions) -> Self {
        Self {
            id: next_view_id(),
            bounds: Rect::default(),
            options,
            dirty: true,
            focused: false,
            title: String::new(),
            buf: Buffer::default(),
            sink: None,
        }
    }

    /// Unique identifier for this view, auto-assigned at creation.
    pub fn id(&self) -> ViewId {
        self.id
    }

    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    pub fn options(&self) -> ViewOptions {
        self.options
    }

    pub fn set_modal(&mut self, modal: bool) {
        self.options.modal = modal;
    }

    pub fn set_preprocess(&mut self, enabled: bool) {
        self.options.preprocess = enabled;
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn set_title(&mut self, t: impl Into<String>) {
        self.title = t.into();
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn is_focused(&self) -> bool {
        self.focused
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn mark_redrawn(&mut self) {
        self.dirty = false;
    }

    pub fn set_focused(&mut self, f: bool) {
        self.focused = f;
    }

    pub fn set_bounds(&mut self, r: Rect) {
        if self.bounds.w != r.w || self.bounds.h != r.h {
            self.buf.resize(r.w, r.h);
            self.bounds = Rect::new(0, 0, r.w, r.h);
            self.dirty = true;
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buf
    }

    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buf
    }

    pub fn set_sink(&mut self, sink: EventSink) {
        self.sink = Some(sink);
    }

    pub fn sink(&self) -> Option<&EventSink> {
        self.sink.as_ref()
    }

    /// Push an event to the owner's sink. No-op if sink not set.
    pub fn put_event(&self, event: Event) {
        if let Some(ref sink) = self.sink {
            sink.push(event);
        }
    }

    /// Push a command event to the owner's sink.
    pub fn put_command(&self, id: CommandId, data: Option<Box<dyn Any + Send>>) {
        self.put_event(Event::Command {
            id,
            data,
            broadcast: false,
        });
    }

    pub fn put_broadcast(&self, id: CommandId, data: Option<Box<dyn Any + Send>>) {
        self.put_event(Event::Command {
            id,
            data,
            broadcast: true,
        });
    }
}

impl Default for ViewState {
    fn default() -> Self {
        Self::new(ViewOptions {
            focusable: true,
            ..ViewOptions::default()
        })
    }
}
