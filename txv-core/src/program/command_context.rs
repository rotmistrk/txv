//! CommandContext — passed to the command handler during dispatch.

use crate::view::{EventSink, View};

/// Context passed to the command handler.
pub struct CommandContext<'a> {
    pub(crate) command: u16,
    pub(crate) data: &'a Option<Box<dyn std::any::Any + Send>>,
    pub(crate) sink: &'a EventSink,
    pub(crate) desktop: &'a mut dyn View,
}

impl<'a> CommandContext<'a> {
    pub fn new(
        command: u16,
        data: &'a Option<Box<dyn std::any::Any + Send>>,
        sink: &'a EventSink,
        desktop: &'a mut dyn View,
    ) -> Self {
        Self {
            command,
            data,
            sink,
            desktop,
        }
    }

    pub fn command(&self) -> u16 {
        self.command
    }

    pub fn data(&self) -> &Option<Box<dyn std::any::Any + Send>> {
        self.data
    }

    pub fn sink(&self) -> &EventSink {
        self.sink
    }

    pub fn desktop_mut(&mut self) -> &mut dyn View {
        self.desktop
    }

    /// Split into (command, data, sink, desktop) for simultaneous access.
    pub fn split(&mut self) -> (u16, &Option<Box<dyn std::any::Any + Send>>, &EventSink, &mut dyn View) {
        (self.command, self.data, self.sink, self.desktop)
    }
}
