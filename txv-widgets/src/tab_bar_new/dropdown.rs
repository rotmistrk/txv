//! TabBar key handling (Alt-digit switching).

use txv_core::prelude::*;

use super::{TabBar, TabBarMode};

impl TabBar {
    /// Handle events (Alt-digit for tab switching).
    pub(crate) fn handle_event(&mut self, event: &Event) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        if !self.handle_keys {
            return HandleResult::Ignored;
        }
        self.handle_alt_key(key)
    }

    fn handle_alt_key(&mut self, key: &KeyEvent) -> HandleResult {
        // Standard Alt-digit
        if key.modifiers().alt() && !key.modifiers().ctrl() && !key.modifiers().shift() {
            if let KeyCode::Char(c) = key.code() {
                if let Some(n) = c.to_digit(10) {
                    return self.try_activate_number(n as usize);
                }
            }
        }
        // macOS Option-digit chars (arrive as plain chars, no ALT modifier)
        if key.modifiers() == KeyMod::NONE {
            if let KeyCode::Char(c) = key.code() {
                if let Some(n) = mac_option_digit(c) {
                    return self.try_activate_number(n);
                }
            }
        }
        HandleResult::Ignored
    }

    fn try_activate_number(&mut self, n: usize) -> HandleResult {
        if self.titles.len() <= 1 {
            return HandleResult::Ignored;
        }
        if n >= 1 && self.can_activate_number(n) {
            let prev = self.active;
            self.activate_by_number(n);
            if self.active != prev {
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
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
            TabBarMode::Lru => self.activate_lru_number(n),
        }
    }

    fn can_activate_number(&self, n: usize) -> bool {
        match self.mode {
            TabBarMode::Single | TabBarMode::Static => n.saturating_sub(1) < self.titles.len(),
            TabBarMode::Lru => {
                let mut count = 0;
                for &i in &self.lru_order {
                    if i == self.active || i >= self.titles.len() {
                        continue;
                    }
                    count += 1;
                    if count == n {
                        return true;
                    }
                }
                false
            }
        }
    }

    fn activate_lru_number(&mut self, n: usize) {
        let mut count = 0;
        for &i in &self.lru_order {
            if i == self.active || i >= self.titles.len() {
                continue;
            }
            count += 1;
            if count == n {
                self.set_active(i);
                return;
            }
        }
    }
}

/// Map macOS Option-digit character to digit (1-9).
pub(crate) fn mac_option_digit(c: char) -> Option<usize> {
    match c {
        '¡' => Some(1),
        '™' => Some(2),
        '£' => Some(3),
        '¢' => Some(4),
        '∞' => Some(5),
        '§' => Some(6),
        '¶' => Some(7),
        '•' => Some(8),
        'ª' => Some(9),
        _ => None,
    }
}
