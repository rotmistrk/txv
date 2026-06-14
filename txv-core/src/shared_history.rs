//! SharedHistory — shared input history ring for InputLine instances.

use std::sync::{Arc, Mutex};

/// Shared history buffer. Multiple InputLines of the same role share one.
#[derive(Clone)]
pub struct SharedHistory {
    entries: Arc<Mutex<Vec<String>>>,
    max: usize,
}

impl SharedHistory {
    pub fn new(max: usize) -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            max,
        }
    }

    pub fn push(&self, text: &str) {
        if text.is_empty() {
            return;
        }
        if let Ok(mut entries) = self.entries.lock() {
            entries.retain(|e| e != text);
            entries.push(text.to_string());
            if entries.len() > self.max {
                entries.remove(0);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.entries.lock().map(|e| e.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn get(&self, index: usize) -> Option<String> {
        let entries = self.entries.lock().ok()?;
        entries.get(index).cloned()
    }
}
