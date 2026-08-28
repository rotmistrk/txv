//! Command registry — maps command IDs to metadata (label, help text).
//!
//! Every command should be registered with a human-readable label and help text.
//! This enables the help system to show meaningful descriptions for all key bindings.

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

use crate::event::CommandId;

/// Metadata for a registered command.
#[derive(Clone, Debug)]
pub struct CommandMeta {
    /// Machine-readable name (e.g., "toggle-tree").
    name: String,
    /// Short human-readable label (e.g., "Toggle tree panel").
    label: String,
    /// Full help description (e.g., "Show or hide the file tree panel").
    help: String,
}

impl CommandMeta {
    pub fn new(name: impl Into<String>, label: impl Into<String>, help: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            help: help.into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn help(&self) -> &str {
        &self.help
    }
}

/// Global command registry.
static REGISTRY: LazyLock<RwLock<HashMap<CommandId, CommandMeta>>> = LazyLock::new(|| RwLock::new(HashMap::new()));

/// Register a command with its metadata.
pub fn register(id: CommandId, meta: CommandMeta) {
    if let Ok(mut reg) = REGISTRY.write() {
        reg.insert(id, meta);
    }
}

/// Look up command metadata by ID.
pub fn lookup(id: CommandId) -> Option<CommandMeta> {
    let guard = REGISTRY.read().ok()?;
    guard.get(&id).cloned()
}

/// Get the label for a command, or a fallback if not registered.
pub fn label(id: CommandId) -> String {
    match lookup(id) {
        Some(m) => m.label,
        None => format!("cmd:{}", id),
    }
}

/// Get the help text for a command, or a fallback if not registered.
pub fn help(id: CommandId) -> String {
    match lookup(id) {
        Some(m) => m.help,
        None => format!("(undocumented command {})", id),
    }
}
