//! Program — the correct way to build a TXV application.
//!
//! Program handles the event loop, three-phase dispatch, draw cycle,
//! resize, and quit. The application only provides:
//! - A desktop view (the main content)
//! - A status bar view (preprocess, key→command translation)
//! - A command handler (what to do when commands arrive)
//!
//! # Example
//!
//! ```ignore
//! use txv_core::prelude::*;
//! use txv_core::program::Program;
//!
//! let desktop = MyDesktop::new();
//! let status = MyStatusBar::new();
//!
//! Program::new(desktop, status)
//!     .run(&mut backend, |ctx| {
//!         match ctx.command {
//!             CM_OPEN_FILE => { /* handle */ }
//!             _ => {}
//!         }
//!     });
//! ```
//!
//! You NEVER manually dispatch events.
//! You NEVER call child.handle() yourself.
//! Program does it all correctly.

use std::time::Duration;

use crate::cell::Style;
use crate::commands::CM_QUIT;
use crate::event::Event;
use crate::geometry::Rect;
use crate::group::GroupState;
use crate::run::Backend;
use crate::view::{EventSink, HandleResult, View, ViewOptions};

/// Context passed to the command handler.
pub struct CommandContext<'a> {
    /// The command ID.
    pub command: u16,
    /// The command data payload.
    pub data: &'a Option<Box<dyn std::any::Any + Send>>,
    /// Event sink to emit new commands.
    pub sink: &'a EventSink,
    /// Access to the desktop (child 1 of the group).
    pub desktop: &'a mut dyn View,
}

/// The TXV application runner. Handles event loop, dispatch, draw.
pub struct Program {
    group: GroupState,
    sink: EventSink,
    quit_requested: bool,
    repaint_requested: bool,
}

impl Program {
    /// Create a new Program with a desktop and status bar.
    ///
    /// The status bar MUST have `preprocess: true` in its ViewOptions.
    /// The desktop is the focused child that receives normal events.
    pub fn new(status_bar: Box<dyn View>, desktop: Box<dyn View>) -> Self {
        let sink = EventSink::new();
        let mut group = GroupState::new(ViewOptions {
            focusable: true,
            ..ViewOptions::default()
        });
        group.set_sink(sink.clone());
        // Child 0: status bar (preprocess — sees keys first)
        group.insert(status_bar);
        // Child 1: desktop (focused — gets normal events)
        group.insert(desktop);
        group.set_focused_index(1);
        group.child_mut(1).unwrap().select();

        Self {
            group,
            sink,
            quit_requested: false,
            repaint_requested: false,
        }
    }

    /// Run the application event loop.
    pub fn run<F>(&mut self, backend: &mut dyn Backend, mut handler: F)
    where
        F: FnMut(&mut CommandContext),
    {
        backend.enter();
        let (w, h) = backend.size();

        // Initial layout
        self.layout(w, h);

        loop {
            // Draw (only if dirty)
            if self.group.any_dirty() {
                self.draw_and_flush(backend);
            }

            // Poll event
            if let Some(event) = backend.poll_event(Duration::from_millis(50)) {
                if let Event::Resize(mut nw, mut nh) = event {
                    while let Some(next) = backend.poll_event(Duration::from_millis(0)) {
                        if let Event::Resize(w2, h2) = next {
                            nw = w2;
                            nh = h2;
                        } else {
                            self.layout(nw, nh);
                            backend.invalidate();
                            self.group.dispatch(&next);
                            nw = 0;
                            break;
                        }
                    }
                    if nw > 0 {
                        self.layout(nw, nh);
                        backend.invalidate();
                    }
                } else {
                    self.group.dispatch(&event);
                }
            } else {
                // Tick
                self.group.dispatch(&Event::Tick);
                self.sink.push(Event::Command {
                    id: crate::commands::CM_TICK,
                    data: None,
                    broadcast: false,
                });
            }

            // Drain and process commands from sink
            if self.drain_commands(&mut handler) {
                break;
            }
            if self.repaint_requested {
                self.repaint_requested = false;
                let (nw, nh) = backend.size();
                self.layout(nw, nh);
                backend.invalidate();
            }
        }

        backend.leave();
    }

    /// Run exactly N iterations of the event loop (for testing).
    pub fn run_cycles(&mut self, backend: &mut dyn Backend, handler: &mut dyn FnMut(&mut CommandContext), n: usize) {
        let (w, h) = backend.size();
        self.layout(w, h);

        for _ in 0..n {
            // Process all pending events
            while let Some(event) = backend.poll_event(Duration::ZERO) {
                if let Event::Resize(nw, nh) = &event {
                    self.layout(*nw, *nh);
                }
                self.group.dispatch(&event);

                if self.drain_commands(handler) {
                    self.draw_and_flush(backend);
                    return;
                }
            }

            // Tick
            self.group.dispatch(&Event::Tick);
            self.drain_commands(handler);

            // Draw and flush
            self.draw_and_flush(backend);
        }
    }

    /// Drain the event sink, dispatch commands. Returns true if CM_QUIT received.
    fn drain_commands(&mut self, handler: &mut dyn FnMut(&mut CommandContext)) -> bool {
        loop {
            let events = self.sink.drain();
            if events.is_empty() {
                return false;
            }
            for ev in events {
                if let Event::Command { id, .. } = &ev {
                    if *id == CM_QUIT {
                        self.quit_requested = true;
                        return true;
                    }
                    if *id == crate::commands::CM_REPAINT {
                        self.repaint_requested = true;
                        continue;
                    }
                }
                // Re-dispatch through the group
                if self.group.dispatch(&ev) == HandleResult::Consumed {
                    continue;
                }
                // Unhandled command → app handler
                if let Event::Command { id, ref data, .. } = ev {
                    let desktop = &mut *self.group.children[1];
                    let mut ctx = CommandContext {
                        command: id,
                        data,
                        sink: &self.sink,
                        desktop,
                    };
                    handler(&mut ctx);
                }
            }
        }
    }

    fn draw_and_flush(&mut self, backend: &mut dyn Backend) {
        for child in &mut self.group.children {
            child.draw();
        }
        self.group.buffer_mut().fill(' ', Style::default());
        // Safety: children (immutable) and buffer (mutable) are disjoint fields of GroupState.
        let buf_ptr = self.group.buffer_mut() as *mut crate::buffer::Buffer;
        for (i, child) in self.group.children.iter().enumerate() {
            let (ox, oy) = self.group.origins.get(i).copied().unwrap_or((0, 0));
            unsafe {
                (*buf_ptr).blit(child.buffer(), ox, oy);
            }
        }
        self.group.mark_redrawn();
        for child in &mut self.group.children {
            child.mark_redrawn();
        }
        backend.flush(self.group.buffer());
        backend.set_cursor(self.group.cursor());
    }

    /// Compute layout: desktop gets all but last row, status gets last row.
    fn layout(&mut self, w: u16, h: u16) {
        let full = Rect::new(0, 0, w, h);
        self.group.set_bounds(full);

        if h >= 2 {
            // Desktop: origin (0,0), size (w, h-1)
            self.group.set_child_origin(1, 0, 0);
            self.group.children[1].set_bounds(Rect::new(0, 0, w, h - 1));
            // Status bar: origin (0, h-1), size (w, 1)
            self.group.set_child_origin(0, 0, h - 1);
            self.group.children[0].set_bounds(Rect::new(0, 0, w, 1));
        } else {
            self.group.set_child_origin(1, 0, 0);
            self.group.children[1].set_bounds(full);
            self.group.set_child_origin(0, 0, 0);
            self.group.children[0].set_bounds(Rect::new(0, 0, 0, 0));
        }
    }

    /// Access the desktop view.
    pub fn desktop(&self) -> &dyn View {
        &*self.group.children[1]
    }

    pub fn desktop_mut(&mut self) -> &mut dyn View {
        &mut *self.group.children[1]
    }

    pub fn status_bar(&self) -> &dyn View {
        &*self.group.children[0]
    }

    pub fn status_bar_mut(&mut self) -> &mut dyn View {
        &mut *self.group.children[0]
    }

    /// Access the event sink (for external command injection).
    pub fn sink(&self) -> &EventSink {
        &self.sink
    }

    /// Insert a named child view (drawn on top in standard cycle).
    pub fn insert_named(&mut self, name: &str, child: Box<dyn View>) {
        self.group.insert_named(name, child);
    }

    /// Remove a named child view.
    pub fn remove_named(&mut self, name: &str) {
        self.group.remove_named(name);
    }

    /// Returns true if CM_QUIT was received during the last run_cycles.
    pub fn should_quit(&self) -> bool {
        self.quit_requested
    }
}
