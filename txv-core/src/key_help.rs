//! KeyHelpEntry — structured key binding description for introspection.

/// A single key binding description.
#[derive(Clone, Debug)]
pub struct KeyHelpEntry {
    key: String,
    action: String,
    group: String,
}

impl KeyHelpEntry {
    pub fn new(key: impl Into<String>, action: impl Into<String>, group: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            action: action.into(),
            group: group.into(),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn action(&self) -> &str {
        &self.action
    }

    pub fn group(&self) -> &str {
        &self.group
    }

    pub fn set_key(&mut self, k: impl Into<String>) {
        self.key = k.into();
    }

    pub fn set_group(&mut self, g: impl Into<String>) {
        self.group = g.into();
    }
}
