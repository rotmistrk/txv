//! ClipEntry — a single clipboard ring entry.

use std::time::Instant;

/// A single clipboard entry.
#[derive(Clone)]
pub struct ClipEntry {
    pub text: String,
    pub source: String,
    pub timestamp: Instant,
    pub line_count: usize,
}
