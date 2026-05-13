//! View trait, ViewState, EventQueue, and the delegate_view_state! macro.

use std::any::Any;

use crate::event::{CommandId, Event};
use crate::geometry::Rect;
use crate::surface::Surface;

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

/// Event queue — views emit commands via `put_command`.
pub struct EventQueue {
    events: Vec<Event>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn put(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn put_command(&mut self, id: CommandId, data: Option<Box<dyn Any + Send>>) {
        self.events.push(Event::Command { id, data });
    }

    pub fn drain(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Default for EventQueue {
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
    fn draw(&self, surface: &mut Surface);
    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult;
    fn select(&mut self) {}
    fn unselect(&mut self) {}
    fn bounds(&self) -> Rect;
    fn set_bounds(&mut self, rect: Rect);
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
    /// Downcast support. Override to return `self`.
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        None
    }
}

/// Common view state — embed in every view.
pub struct ViewState {
    bounds: Rect,
    pub options: ViewOptions,
    dirty: bool,
    focused: bool,
    pub title: String,
}

impl ViewState {
    pub fn new(options: ViewOptions) -> Self {
        Self {
            bounds: Rect::default(),
            options,
            dirty: true,
            focused: false,
            title: String::new(),
        }
    }

    pub fn bounds(&self) -> Rect {
        self.bounds
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
        self.bounds = r;
        self.dirty = true;
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
