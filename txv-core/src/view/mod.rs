//! View trait, ViewState, EventSink, and the delegate_view_state! macro.

mod event_sink;
mod view_options;
mod view_state;

use crate::buffer::Buffer;
use crate::event::Event;
use crate::geometry::Rect;

pub use event_sink::EventSink;
pub use view_options::ViewOptions;
pub use view_state::ViewState;

/// Unique view identifier, auto-assigned at creation.
pub type ViewId = u64;

/// Result of handling an event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HandleResult {
    Consumed,
    Ignored,
}

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
        false
    }
    fn mark_redrawn(&mut self) {}

    /// Check if redraw needed, draw if so, clear dirty. Returns true if drew.
    fn render(&mut self) -> bool {
        if !self.needs_redraw() {
            return false;
        }
        self.draw();
        self.mark_redrawn();
        true
    }
    /// Called before closing. Return Ok to allow, Denied to prevent.
    fn can_close(&self) -> CloseResult {
        CloseResult::Ok
    }
    /// Downcast support (immutable). Override to return `self`.
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        None
    }
    /// Downcast support. Override to return `self`.
    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
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
    /// If this view owns a GroupState, expose it for coordinate queries.
    fn group_state(&self) -> Option<&crate::group::GroupState> {
        None
    }
}

// View delegation macros are in view_macros.rs
#[path = "../view_macros.rs"]
mod _view_macros;
