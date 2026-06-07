//! Message — structured application message with severity and origin.

use std::time::{SystemTime, UNIX_EPOCH};

/// Severity level for messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MsgLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl MsgLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Debug => "DBG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERR",
        }
    }
}

/// A single application message.
#[derive(Clone)]
pub struct Message {
    pub(crate) level: MsgLevel,
    pub(crate) origin: &'static str,
    pub(crate) text: String,
    pub(crate) timestamp: u64,
    pub(crate) count: u32,
}

impl Message {
    pub fn new(level: MsgLevel, origin: &'static str, text: impl Into<String>) -> Self {
        Self {
            level,
            origin,
            text: text.into(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            count: 1,
        }
    }

    pub fn info(origin: &'static str, text: impl Into<String>) -> Self {
        Self::new(MsgLevel::Info, origin, text)
    }

    pub fn warn(origin: &'static str, text: impl Into<String>) -> Self {
        Self::new(MsgLevel::Warn, origin, text)
    }

    pub fn error(origin: &'static str, text: impl Into<String>) -> Self {
        Self::new(MsgLevel::Error, origin, text)
    }

    pub fn level(&self) -> MsgLevel {
        self.level
    }

    pub fn origin(&self) -> &'static str {
        self.origin
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn count(&self) -> u32 {
        self.count
    }

    pub fn increment(&mut self) {
        self.count += 1;
    }
}
