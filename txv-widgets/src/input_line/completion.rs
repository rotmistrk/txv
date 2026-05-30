//! InputLine completion and sidekick integration.

use std::sync::{Arc, Mutex};

use txv_core::prelude::*;

use super::completion_list::{CompletionItem, CompletionList};
use super::InputLine;
use crate::list_view::ListView;
use crate::sidekick::{SidekickShow, CM_SIDEKICK_HIDE, CM_SIDEKICK_SHOW};

impl InputLine {
    pub(crate) fn try_complete(&mut self) {
        let Some(ref completer) = self.completer else {
            return;
        };
        let byte_cursor = self.char_to_byte(self.cursor);
        let mut items: Vec<CompletionItem> = Vec::new();
        let _ = completer.complete(&self.text, byte_cursor, &mut |c| {
            items.push(CompletionItem::new(c.text().to_string(), c.display().to_string()));
            Ok(items.len() < 20)
        });
        match items.len() {
            0 => self.hide_sidekick(),
            1 => {
                self.text = items.remove(0).text().to_string();
                self.cursor = self.char_count();
                self.hide_sidekick();
                self.update_width();
            }
            _ => self.show_sidekick(items),
        }
    }

    /// Update sidekick with current completions (called on text change).
    pub(crate) fn update_completions(&mut self) {
        let Some(ref completer) = self.completer else {
            self.hide_sidekick();
            return;
        };
        let byte_cursor = self.char_to_byte(self.cursor);
        let mut items: Vec<CompletionItem> = Vec::new();
        let _ = completer.complete(&self.text, byte_cursor, &mut |c| {
            items.push(CompletionItem::new(c.text().to_string(), c.display().to_string()));
            Ok(items.len() < 20)
        });
        if items.len() > 1 {
            self.show_sidekick(items);
        } else {
            self.hide_sidekick();
        }
    }

    fn show_sidekick(&mut self, items: Vec<CompletionItem>) {
        let list = CompletionList::new(items);
        let max_w = list.max_display_width();
        if let Ok(mut lv) = self.popup.lock() {
            *lv = ListView::new(list);
        }
        self.sidekick_visible = true;
        self.emit_sidekick_show(max_w);
    }

    pub(crate) fn hide_sidekick(&mut self) {
        if self.sidekick_visible {
            self.sidekick_visible = false;
            self.state.put_command(CM_SIDEKICK_HIDE, None);
        }
    }

    fn emit_sidekick_show(&self, content_width: usize) {
        let h = self.popup.lock().map(|lv| lv.data().len()).unwrap_or(0).min(8) as u16;
        let w = (content_width as u16 + 2).clamp(10, 40);
        // Rect relative to self: x=0, y=0 (will be placed above emitter by manager).
        let rect = Rect::new(0, 0, w, h);
        let data = SidekickShow {
            rect,
            view: Arc::clone(&self.popup) as Arc<Mutex<dyn View>>,
            emitter_id: self.state.id(),
        };
        self.state.put_command(CM_SIDEKICK_SHOW, Some(Box::new(data)));
    }

    pub(crate) fn sidekick_select_next(&mut self) {
        if let Ok(mut lv) = self.popup.lock() {
            lv.select_next();
        }
    }

    pub(crate) fn sidekick_select_prev(&mut self) {
        if let Ok(mut lv) = self.popup.lock() {
            lv.select_prev();
        }
    }

    /// Apply the currently selected completion item.
    pub(crate) fn apply_sidekick_selection(&mut self) {
        let text = self
            .popup
            .lock()
            .ok()
            .and_then(|lv| lv.data().selected_text(lv.cursor()).map(String::from));
        if let Some(t) = text {
            self.text = t;
            self.cursor = self.char_count();
            self.update_width();
        }
        self.hide_sidekick();
    }
}
