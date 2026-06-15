//! Insert mode exit and block insert replication.

use super::keymap::EditorMode;
use super::Editor;

impl Editor {
    pub(super) fn exit_insert(&mut self) {
        if let Some((sl, el, sc)) = self.pending_block_insert.take() {
            self.replicate_block_insert(sl, el, sc);
        }
        self.buf().end_group();
        self.mode = EditorMode::Normal;
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
        }
    }

    fn replicate_block_insert(&mut self, sl: usize, el: usize, sc: usize) {
        let inserted_len = self.cursor_col.saturating_sub(sc);
        if inserted_len == 0 {
            return;
        }
        let line = self.buf().line(sl).unwrap_or_default();
        let inserted: String = line.chars().skip(sc).take(inserted_len).collect();
        for line_idx in (sl + 1..=el).rev() {
            if line_idx >= self.buf().line_count() {
                continue;
            }
            let offset = self.buf().line_col_to_offset(line_idx, sc).unwrap_or(0);
            self.buf().insert(offset, &inserted);
        }
    }
}
