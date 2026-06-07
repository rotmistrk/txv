//! TreeTableView key handling — vim navigation, structural ops, undo, column focus.

use txv_core::prelude::*;

use super::TreeTableView;
use crate::tree_table_source::TreeTableSource;

impl<D: TreeTableSource> TreeTableView<D> {
    /// Handle vim-style and structural keys. Returns Consumed if handled.
    pub(super) fn handle_char_key(&mut self, key: &KeyEvent) -> HandleResult {
        let KeyCode::Char(ch) = key.code() else {
            return self.handle_special_key(key);
        };
        match ch {
            'j' => {
                self.move_down();
                HandleResult::Consumed
            }
            'k' => {
                self.move_up();
                HandleResult::Consumed
            }
            'g' => self.jump_to_start(),
            'G' => self.jump_to_end(),
            ' ' | 'l' => {
                self.handle_enter_right();
                HandleResult::Consumed
            }
            'h' => {
                self.handle_left();
                HandleResult::Consumed
            }
            'n' => self.do_structural_op(|d, r| d.add_sibling(r)),
            'b' => self.do_structural_op(|d, r| d.add_child(r)),
            'd' => self.do_structural_op(|d, r| d.delete(r)),
            'J' => self.do_structural_op(|d, r| d.swap_down(r)),
            'K' => self.do_structural_op(|d, r| d.swap_up(r)),
            'H' => self.do_structural_op(|d, r| d.promote(r)),
            'L' => self.do_structural_op(|d, r| d.demote(r)),
            'u' => {
                if self.data.undo() {
                    self.clamp_cursor();
                    self.state.mark_dirty();
                }
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }

    fn jump_to_start(&mut self) -> HandleResult {
        self.cursor = 0;
        self.sync_scroll();
        self.state.mark_dirty();
        HandleResult::Consumed
    }

    fn jump_to_end(&mut self) -> HandleResult {
        self.cursor = self.data.visible_count().saturating_sub(1);
        self.sync_scroll();
        self.state.mark_dirty();
        HandleResult::Consumed
    }

    fn handle_special_key(&mut self, key: &KeyEvent) -> HandleResult {
        match key.code() {
            KeyCode::Tab => {
                self.cycle_focused_col();
                HandleResult::Consumed
            }
            KeyCode::Left => {
                if self.h_scroll > 0 {
                    self.h_scroll = self.h_scroll.saturating_sub(4);
                    self.state.mark_dirty();
                }
                HandleResult::Consumed
            }
            KeyCode::Right => {
                self.h_scroll += 4;
                self.state.mark_dirty();
                HandleResult::Consumed
            }
            KeyCode::Char('r') if key.modifiers().ctrl() => {
                if self.data.redo() {
                    self.clamp_cursor();
                    self.state.mark_dirty();
                }
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }

    fn do_structural_op(&mut self, op: fn(&mut D, usize) -> Option<usize>) -> HandleResult {
        let row = self.cursor;
        if row >= self.data.visible_count() {
            return HandleResult::Consumed;
        }
        self.data.save_snapshot();
        if let Some(new_pos) = op(&mut self.data, row) {
            self.set_cursor(new_pos);
        }
        self.state.mark_dirty();
        HandleResult::Consumed
    }

    fn cycle_focused_col(&mut self) {
        let total_cols = self.data.column_count() + 1; // col 0 = tree, 1..N = extra
        let start = self.focused_col.map_or(0, |c| c + 1);
        for i in 0..total_cols {
            let col = (start + i) % total_cols;
            if self.data.column_validator(col).is_some() {
                self.focused_col = Some(col);
                self.state.mark_dirty();
                return;
            }
        }
    }

    fn clamp_cursor(&mut self) {
        let max = self.data.visible_count().saturating_sub(1);
        if self.cursor > max {
            self.cursor = max;
        }
        self.sync_scroll();
    }
}
