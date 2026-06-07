//! ClipEntry — a single clipboard ring entry.

use std::time::Instant;

/// A single clipboard entry.
#[derive(Clone)]
pub struct ClipEntry {
    pub(crate) text: String,
    pub(crate) source: String,
    pub(crate) timestamp: Instant,
    pub(crate) line_count: usize,
}

impl ClipEntry {
    pub fn new(text: String, source: String) -> Self {
        let line_count = text.lines().count().max(1);
        Self {
            text,
            source,
            timestamp: Instant::now(),
            line_count,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn timestamp(&self) -> Instant {
        self.timestamp
    }

    pub fn line_count(&self) -> usize {
        self.line_count
    }
}
