//! TabBar dropdown (searchable) and key handling.

use txv_core::prelude::*;

use super::{TabBar, TabBarMode};

impl TabBar {
    /// Open the dropdown.
    pub fn open_dropdown(&mut self) {
        self.dropdown_cursor = Some(0);
        self.dropdown_filter.clear();
        self.state.mark_dirty();
    }

    /// Close the dropdown.
    pub fn close_dropdown(&mut self) {
        if self.dropdown_cursor.is_some() {
            self.dropdown_cursor = None;
            self.dropdown_filter.clear();
            self.state.mark_dirty();
        }
    }

    /// Whether dropdown is open.
    pub fn dropdown_open(&self) -> bool {
        self.dropdown_cursor.is_some()
    }

    /// Filtered tab list for dropdown display.
    /// Returns (tab_index, label, numbered).
    pub(crate) fn dropdown_entries(&self) -> Vec<(usize, String, bool)> {
        let order = self.display_order_for_dropdown();
        let query = self.dropdown_filter.to_lowercase();
        order
            .into_iter()
            .filter(|(_, label, _)| query.is_empty() || fuzzy_match(label, &query))
            .collect()
    }

    /// Display order for dropdown with numbering per mode.
    fn display_order_for_dropdown(&self) -> Vec<(usize, String, bool)> {
        match self.mode {
            TabBarMode::Single | TabBarMode::Static => (0..self.titles.len())
                .map(|i| {
                    let num = i + 1;
                    (i, format!("{num}:{}", self.titles[i]), true)
                })
                .collect(),
            TabBarMode::Lru => {
                let mut entries = Vec::new();
                // Active first, no number
                entries.push((self.active, self.titles[self.active].clone(), false));
                // Rest by LRU order, numbered
                let mut num = 1;
                for &i in &self.lru_order {
                    if i != self.active && i < self.titles.len() {
                        entries.push((i, format!("{num}:{}", self.titles[i]), true));
                        num += 1;
                    }
                }
                entries
            }
        }
    }

    /// Handle events (keys + dropdown).
    pub(crate) fn handle_event(&mut self, event: &Event) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };

        // Dropdown open — consume all keys
        if self.dropdown_cursor.is_some() {
            return self.handle_dropdown_key(key);
        }

        if !self.handle_keys {
            return HandleResult::Ignored;
        }

        // M-0: open dropdown
        if key.modifiers.alt && !key.modifiers.ctrl && !key.modifiers.shift {
            if let KeyCode::Char('0') = key.code {
                self.open_dropdown();
                return HandleResult::Consumed;
            }
            // M-digit: activate tab
            if let KeyCode::Char(c) = key.code {
                if let Some(n) = c.to_digit(10) {
                    if n >= 1 {
                        self.activate_by_number(n as usize);
                        return HandleResult::Consumed;
                    }
                }
            }
        }

        HandleResult::Ignored
    }

    fn handle_dropdown_key(&mut self, key: &KeyEvent) -> HandleResult {
        let Some(cursor) = self.dropdown_cursor else {
            return HandleResult::Ignored;
        };
        match key.code {
            KeyCode::Esc => {
                self.dropdown_cursor = None;
                self.dropdown_filter.clear();
                self.state.mark_dirty();
            }
            KeyCode::Enter => {
                let entries = self.dropdown_entries();
                if let Some(&(tab_idx, _, _)) = entries.get(cursor) {
                    self.set_active(tab_idx);
                }
                self.dropdown_cursor = None;
                self.dropdown_filter.clear();
                self.state.mark_dirty();
            }
            KeyCode::Down => {
                let count = self.dropdown_entries().len();
                if count > 0 {
                    self.dropdown_cursor = Some((cursor + 1) % count);
                    self.state.mark_dirty();
                }
            }
            KeyCode::Up => {
                let count = self.dropdown_entries().len();
                if count > 0 {
                    self.dropdown_cursor = Some(if cursor == 0 {
                        count - 1
                    } else {
                        cursor - 1
                    });
                    self.state.mark_dirty();
                }
            }
            KeyCode::Backspace => {
                self.dropdown_filter.pop();
                let count = self.dropdown_entries().len();
                if cursor >= count && count > 0 {
                    self.dropdown_cursor = Some(count - 1);
                }
                self.state.mark_dirty();
            }
            KeyCode::Char(c) if !key.modifiers.ctrl && !key.modifiers.alt => {
                self.dropdown_filter.push(c);
                self.dropdown_cursor = Some(0);
                self.state.mark_dirty();
            }
            _ => {}
        }
        HandleResult::Consumed
    }

    /// Activate tab by M-digit number (1-based).
    /// Static mode: M-1→tab 0, M-2→tab 1, etc.
    /// LRU mode: M-1→most recent other, M-2→next recent, etc.
    pub fn activate_by_number(&mut self, n: usize) {
        match self.mode {
            TabBarMode::Single | TabBarMode::Static => {
                let idx = n.saturating_sub(1);
                if idx < self.titles.len() {
                    self.set_active(idx);
                }
            }
            TabBarMode::Lru => {
                // n=1 is first inactive in LRU order
                let mut count = 0;
                for &i in &self.lru_order {
                    if i != self.active && i < self.titles.len() {
                        count += 1;
                        if count == n {
                            self.set_active(i);
                            return;
                        }
                    }
                }
            }
        }
    }
}

/// Simple fuzzy match: all chars of query appear in order in target.
fn fuzzy_match(target: &str, query: &str) -> bool {
    let mut chars = query.chars();
    let mut next = chars.next();
    for c in target.chars().flat_map(|c| c.to_lowercase()) {
        if let Some(q) = next {
            if c == q {
                next = chars.next();
            }
        } else {
            return true;
        }
    }
    next.is_none()
}
