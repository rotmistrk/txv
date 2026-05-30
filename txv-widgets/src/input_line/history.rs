//! InputLine history navigation.

use super::InputLine;

impl InputLine {
    pub(crate) fn push_history(&mut self) {
        if !self.text.is_empty() {
            self.history.push(self.text.clone());
        }
        self.history_pos = None;
    }

    pub(crate) fn handle_history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let pos = match self.history_pos {
            Some(p) => p.saturating_sub(1),
            None => self.history.len() - 1,
        };
        self.history_pos = Some(pos);
        self.text = self.history[pos].clone();
        self.cursor = self.char_count();
        self.selection = None;
        self.update_width();
    }

    pub(crate) fn handle_history_down(&mut self) {
        let Some(pos) = self.history_pos else {
            return;
        };
        if pos + 1 < self.history.len() {
            self.history_pos = Some(pos + 1);
            self.text = self.history[pos + 1].clone();
        } else {
            self.history_pos = None;
            self.text.clear();
        }
        self.cursor = self.char_count();
        self.selection = None;
        self.update_width();
    }
}
