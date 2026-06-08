//! Ex command completion — Tab-cycling in command mode.

use crate::editor::ex_commands::CMD_TABLE_NAMES;

/// State for cycling through completions in command mode.
pub struct ExCompleter {
    /// Completions for the current prefix.
    matches: Vec<String>,
    /// Current cycle index.
    index: usize,
    /// The original prefix before any completion was applied.
    original: String,
}

impl ExCompleter {
    pub fn new() -> Self {
        Self {
            matches: Vec::new(),
            index: 0,
            original: String::new(),
        }
    }

    /// Compute completions for a command buffer. Strips range prefix, completes command word.
    /// `extra_commands` are app-specific command names to include.
    pub fn complete(&mut self, buf: &str, extra_commands: &[&str]) -> Option<&str> {
        // If we're already cycling (same full buffer as last completion result), advance
        if !self.matches.is_empty() && self.is_cycling(buf) {
            return self.cycle_next();
        }
        // Fresh completion
        let cmd_part = strip_range_prefix(buf);
        self.original = buf.to_string();
        self.matches = collect_matches(cmd_part, extra_commands);
        self.index = 0;
        self.matches.first().map(|s| s.as_str())
    }

    /// Check if the current buffer matches what we last completed to.
    fn is_cycling(&self, buf: &str) -> bool {
        if self.matches.is_empty() {
            return false;
        }
        let expected = self.build_result(&self.original, &self.matches[self.index]);
        expected == buf
    }

    /// Cycle to next completion. Returns None if no completions.
    fn cycle_next(&mut self) -> Option<&str> {
        if self.matches.is_empty() {
            return None;
        }
        self.index = (self.index + 1) % self.matches.len();
        Some(&self.matches[self.index])
    }

    /// Reset state (called on any non-Tab keystroke in command mode).
    pub fn reset(&mut self) {
        self.matches.clear();
        self.index = 0;
        self.original.clear();
    }

    /// Build a full command buffer by re-attaching the range prefix.
    pub fn build_result(&self, original_buf: &str, completed_cmd: &str) -> String {
        let cmd_part = strip_range_prefix(original_buf);
        let prefix_len = original_buf.len() - cmd_part.len();
        let range_prefix = &original_buf[..prefix_len];
        format!("{range_prefix}{completed_cmd}")
    }
}

impl Default for ExCompleter {
    fn default() -> Self {
        Self::new()
    }
}

/// Strip the range prefix from a command buffer, returning the command word portion.
/// Examples: "'<,'>s" → "s", "%!sort" → "!sort", "2,+3d" → "d", "set" → "set"
fn strip_range_prefix(buf: &str) -> &str {
    let bytes = buf.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            // Range components: digits, marks, dots, dollar, comma, plus, minus, percent
            b'0'..=b'9' | b'.' | b'$' | b',' | b'+' | b'-' | b'%' => i += 1,
            b'\'' => {
                // Mark reference like '< or '>
                i += 1;
                if i < bytes.len() {
                    i += 1;
                }
            }
            _ => break,
        }
    }
    &buf[i..]
}

fn collect_matches(prefix: &str, extra_commands: &[&str]) -> Vec<String> {
    if prefix.is_empty() {
        return Vec::new();
    }
    let mut results: Vec<String> = CMD_TABLE_NAMES
        .iter()
        .chain(extra_commands.iter())
        .filter(|cmd| cmd.starts_with(prefix))
        .map(|s| s.to_string())
        .collect();
    results.sort();
    results.dedup();
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_simple_command() {
        assert_eq!(strip_range_prefix("set"), "set");
    }

    #[test]
    fn strip_percent_bang() {
        assert_eq!(strip_range_prefix("%!sort"), "!sort");
    }

    #[test]
    fn strip_visual_marks() {
        assert_eq!(strip_range_prefix("'<,'>s"), "s");
    }

    #[test]
    fn strip_numeric_range() {
        assert_eq!(strip_range_prefix("2,+3d"), "d");
    }

    #[test]
    fn complete_cycles() {
        let mut c = ExCompleter::new();
        let result = c.complete("s", &[]);
        assert!(result.is_some());
        let first = result.unwrap().to_string();
        // Cycle
        let result2 = c.complete("s", &[]);
        assert!(result2.is_some());
        // Eventually cycles back
        let mut found_first_again = false;
        for _ in 0..20 {
            if c.complete("s", &[]) == Some(first.as_str()) {
                found_first_again = true;
                break;
            }
        }
        assert!(found_first_again);
    }

    #[test]
    fn complete_with_extra_commands() {
        let mut c = ExCompleter::new();
        let result = c.complete("gre", &["grep", "git"]);
        assert_eq!(result, Some("grep"));
    }
}
