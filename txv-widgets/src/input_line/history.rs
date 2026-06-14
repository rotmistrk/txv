//! InputLine history navigation — uses shared history when available.

use super::InputLine;

impl InputLine {
    pub(crate) fn push_history(&mut self) {
        if self.text.is_empty() {
            self.history_pos = None;
            return;
        }
        if let Some(ref sh) = self.shared_history {
            sh.push(&self.text);
        } else {
            self.history.retain(|e| e != &self.text);
            self.history.push(self.text.clone());
        }
        self.history_pos = None;
    }

    pub(crate) fn handle_history_up(&mut self) {
        let len = self.history_len();
        if len == 0 {
            return;
        }
        let pos = match self.history_pos {
            Some(p) => p.saturating_sub(1),
            None => len - 1,
        };
        self.history_pos = Some(pos);
        if let Some(entry) = self.history_get(pos) {
            self.text = entry;
            self.cursor = self.char_count();
            self.selection = None;
            self.update_width();
        }
    }

    pub(crate) fn handle_history_down(&mut self) {
        let Some(pos) = self.history_pos else {
            return;
        };
        let len = self.history_len();
        if pos + 1 < len {
            self.history_pos = Some(pos + 1);
            if let Some(entry) = self.history_get(pos + 1) {
                self.text = entry;
            }
        } else {
            self.history_pos = None;
            self.text.clear();
        }
        self.cursor = self.char_count();
        self.selection = None;
        self.update_width();
    }

    fn history_len(&self) -> usize {
        if let Some(ref sh) = self.shared_history {
            sh.len()
        } else {
            self.history.len()
        }
    }

    fn history_get(&self, idx: usize) -> Option<String> {
        if let Some(ref sh) = self.shared_history {
            sh.get(idx)
        } else {
            self.history.get(idx).cloned()
        }
    }
}
