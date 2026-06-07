//! InputLine completion and sidekick integration.

use std::sync::{Arc, Mutex};

use txv_core::prelude::*;

use super::completion_item::CompletionItem;
use super::completion_list::CompletionList;
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
            0 => self.show_sidekick(items),
            1 => {
                self.text = items.remove(0).text().to_string();
                self.cursor = self.char_count();
                self.update_width();
                // If completed to a directory (ends with /), show contents
                if self.text.ends_with('/') {
                    self.update_completions();
                } else {
                    self.hide_sidekick();
                }
            }
            _ => {
                // Expand to longest common prefix of all matches
                let lcp = Self::longest_common_prefix(&items);
                if lcp.len() > self.text.len() {
                    self.text = lcp;
                    self.cursor = self.char_count();
                    self.update_width();
                    if self.text.ends_with('/') {
                        self.update_completions();
                        return;
                    }
                }
                self.show_sidekick(items);
            }
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
        self.show_sidekick(items);
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
        let count = self.popup.lock().map(|lv| lv.data().len()).unwrap_or(0);
        // Update count on the frame
        if let Ok(mut frame) = self.popup_frame.lock() {
            frame.set_count(count);
        }
        let content_h = count.min(8) as u16;
        // +2 for top/bottom border
        let h = content_h + 2;
        let w = (content_width as u16 + 4).clamp(14, 42);
        let rect = Rect::new(0, 0, w, h);
        let data = SidekickShow {
            rect,
            view: Arc::clone(&self.popup_frame) as Arc<Mutex<dyn View>>,
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
    fn longest_common_prefix(items: &[CompletionItem]) -> String {
        let first = items[0].text();
        let mut len = first.len();
        for item in &items[1..] {
            len = first
                .bytes()
                .zip(item.text().bytes())
                .take(len)
                .take_while(|(a, b)| a == b)
                .count();
        }
        first[..len].to_string()
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
        // If completed to a directory, show its contents
        if self.text.ends_with('/') {
            self.update_completions();
        } else {
            self.hide_sidekick();
        }
    }
}
