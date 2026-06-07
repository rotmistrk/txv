//! EventSink — shared event queue between views and owner.

use std::any::Any;
use std::mem;
use std::sync::{Arc, Mutex};

use crate::event::{CommandId, Event};

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
        self.push(Event::Command {
            id,
            data,
            broadcast: false,
        });
    }

    pub fn push_broadcast(&self, id: CommandId, data: Option<Box<dyn Any + Send>>) {
        self.push(Event::Command {
            id,
            data,
            broadcast: true,
        });
    }

    pub fn drain(&self) -> Vec<Event> {
        mem::take(&mut *self.events.lock().unwrap_or_else(|e| e.into_inner()))
    }
}

impl Default for EventSink {
    fn default() -> Self {
        Self::new()
    }
}
