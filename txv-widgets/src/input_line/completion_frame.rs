//! CompletionFrame — Group that draws a border around a ListView child.
//!
//! Handles CM_SIDEKICK_NEXT/PREV to navigate, CM_SIDEKICK_APPLY to confirm selection.

use txv_core::palette::palette;
use txv_core::prelude::*;

use super::completion_list::CompletionList;
use crate::list_view::ListView;
use crate::sidekick::{CM_SIDEKICK_APPLY, CM_SIDEKICK_NEXT, CM_SIDEKICK_PREV, CM_SIDEKICK_RESULT};

/// A Group view: border chrome + ListView<CompletionList> as child 0.
pub(crate) struct CompletionFrame {
    group: GroupState,
    count: usize,
}

impl CompletionFrame {
    pub fn new(list: ListView<CompletionList>, count: usize) -> Self {
        let mut group = GroupState::new(ViewOptions::default());
        group.insert(Box::new(list));
        Self { group, count }
    }

    fn list_mut(&mut self) -> Option<&mut ListView<CompletionList>> {
        self.group
            .child_mut(0)
            .and_then(|c| c.as_any_mut())
            .and_then(|a| a.downcast_mut())
    }
}

impl View for CompletionFrame {
    delegate_group_state!(group, override { draw, handle, set_bounds });

    fn set_bounds(&mut self, r: Rect) {
        self.group.set_bounds(r);
        // Child (ListView) fills inside the border
        if r.w() > 2 && r.h() > 2 {
            self.group.set_child_bounds(0, Rect::new(1, 1, r.w() - 2, r.h() - 2));
        }
    }

    fn draw(&mut self) {
        let w = self.group.buffer_mut().width();
        let h = self.group.buffer_mut().height();
        if w < 4 || h < 2 {
            return;
        }
        let style = palette().style(StyleId::Border);
        let bg = palette().style(StyleId::StatusBar);
        self.group.buffer_mut().fill(' ', bg);
        self.group.buffer_mut().draw_box(0, 0, w, h, false, style);
        let label = format!(" {} ", self.count);
        let x = w.saturating_sub(label.len() as u16 + 1);
        self.group.buffer_mut().print(x, 0, &label, style);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Command { id, .. } = event else {
            return HandleResult::Ignored;
        };
        match *id {
            CM_SIDEKICK_NEXT => {
                if let Some(lv) = self.list_mut() {
                    lv.select_next();
                }
                self.group.mark_dirty();
                HandleResult::Consumed
            }
            CM_SIDEKICK_PREV => {
                if let Some(lv) = self.list_mut() {
                    lv.select_prev();
                }
                self.group.mark_dirty();
                HandleResult::Consumed
            }
            CM_SIDEKICK_APPLY => {
                let text = self.list_mut().and_then(|lv| {
                    let cur = lv.cursor();
                    lv.data().selected_text(cur).map(String::from)
                });
                if let Some(t) = text {
                    self.group.put_command(CM_SIDEKICK_RESULT, Some(Box::new(t)));
                }
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}
