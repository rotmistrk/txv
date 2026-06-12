//! TabBar display helpers — order, labels, and style selection.

use super::tab_style::TabStyle;
use super::types::{TabBarMode, SUBSCRIPTS};
use super::TabBar;

impl TabBar {
    /// Get display order based on mode.
    pub(crate) fn display_order(&self) -> Vec<usize> {
        match self.mode {
            TabBarMode::Single => vec![self.active],
            TabBarMode::Static => (0..self.titles.len()).collect(),
            TabBarMode::Lru => {
                let mut order = vec![self.active];
                for &i in &self.lru_order {
                    if i != self.active && i < self.titles.len() {
                        order.push(i);
                    }
                }
                order
            }
        }
    }

    /// Number label for a tab at display position.
    pub(crate) fn number_label(&self, display_pos: usize, tab_idx: usize) -> Option<char> {
        match self.mode {
            TabBarMode::Single => None,
            TabBarMode::Static => {
                if display_pos < 9 {
                    Some(SUBSCRIPTS[display_pos + 1])
                } else {
                    None
                }
            }
            TabBarMode::Lru => {
                if tab_idx == self.active {
                    None
                } else if display_pos > 0 && display_pos <= 9 {
                    Some(SUBSCRIPTS[display_pos])
                } else {
                    None
                }
            }
        }
    }

    /// Tab style for a display position.
    pub(crate) fn tab_style(&self, display_pos: usize, tab_idx: usize) -> TabStyle {
        if tab_idx == self.active {
            if self.focused {
                self.palette.active_focused
            } else {
                self.palette.active_unfocused
            }
        } else {
            // Gradient based on distance from active tab
            let active_pos = match self.mode {
                TabBarMode::Lru => 0, // active is always first in LRU
                _ => self.active,
            };
            let distance = display_pos.abs_diff(active_pos);
            let idx = distance.saturating_sub(1).min(9);
            self.palette.inactive[idx]
        }
    }
}
