//! View trait, ViewState, EventSink, and the delegate_view_state! macro.

use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::buffer::Buffer;
use crate::event::{CommandId, Event};
use crate::geometry::Rect;

/// Unique view identifier, auto-assigned at creation.
pub type ViewId = u64;

static NEXT_VIEW_ID: AtomicU64 = AtomicU64::new(1);

fn next_view_id() -> ViewId {
    NEXT_VIEW_ID.fetch_add(1, Ordering::Relaxed)
}

/// Options flags for a View.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ViewOptions {
    pub preprocess: bool,
    pub postprocess: bool,
    pub focusable: bool,
    pub modal: bool,
}

/// Result of handling an event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HandleResult {
    Consumed,
    Ignored,
}

/// Shared event sink — views push events here, owner drains them.
#[derive(Clone)]
pub struct EventSink {
    events: Arc<Mutex<Vec<Event>>>,
}

impl EventSink {
    pub fn new() -> Self {
        Self {
            events: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn push(&self, event: Event) {
        self.events.lock().unwrap_or_else(|e| e.into_inner()).push(event);
    }

    pub fn push_command(&self, id: CommandId, data: Option<Box<dyn Any + Send>>) {
        self.push(Event::Command { id, data });
    }

    pub fn drain(&self) -> Vec<Event> {
        std::mem::take(&mut *self.events.lock().unwrap_or_else(|e| e.into_inner()))
    }
}

impl Default for EventSink {
    fn default() -> Self {
        Self::new()
    }
}

/// A rectangular UI element.
/// Result of asking a view if it can be closed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloseResult {
    /// Tab can be closed immediately.
    Ok,
    /// Tab refuses to close (reason shown in status bar).
    Denied(String),
}

pub trait View: Send {
    /// Draw into own buffer at relative coords (0,0). Called by parent.
    fn draw(&mut self);
    fn handle(&mut self, event: &Event) -> HandleResult;
    fn select(&mut self) {}
    fn unselect(&mut self) {}
    fn bounds(&self) -> Rect;
    fn set_bounds(&mut self, rect: Rect);
    fn set_sink(&mut self, sink: EventSink);
    /// Unique view identifier, auto-assigned at creation.
    fn view_id(&self) -> ViewId {
        0
    }
    fn options(&self) -> ViewOptions {
        ViewOptions {
            focusable: true,
            ..ViewOptions::default()
        }
    }
    fn title(&self) -> &str {
        ""
    }
    /// Dynamic subtitle (e.g. OSC window title from PTY). Appended to tab name.
    fn subtitle(&self) -> &str {
        ""
    }
    fn needs_redraw(&self) -> bool {
        true
    }
    fn mark_redrawn(&mut self) {}
    /// Called before closing. Return Ok to allow, Denied to prevent.
    fn can_close(&self) -> CloseResult {
        CloseResult::Ok
    }
    /// Downcast support (immutable). Override to return `self`.
    fn as_any(&self) -> Option<&dyn Any> {
        None
    }
    /// Downcast support. Override to return `self`.
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        None
    }
    /// Hardware cursor request (position relative to own bounds).
    fn cursor(&self) -> Option<crate::cursor::CursorRequest> {
        None
    }
    /// Set the palette for this view. Called by parent to propagate style context.
    fn set_palette(&mut self, _palette: std::sync::Arc<dyn crate::palette::Palette>) {}
    /// Access the view's buffer after draw().
    fn buffer(&self) -> &Buffer;
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
        self.put_event(Event::Command { id, data });
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

// View delegation macros are in view_macros.rs
#[path = "view_macros.rs"]
mod _view_macros;
