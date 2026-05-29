//! InputLine completion and sidekick integration.

use std::sync::{Arc, Mutex};

use txv_core::prelude::*;

use super::InputLine;
use crate::sidekick::{SidekickShow, CM_SIDEKICK_HIDE, CM_SIDEKICK_SHOW};

impl InputLine {
    pub(crate) fn try_complete(&mut self) {
        let Some(ref completer) = self.completer else {
            return;
        };
        let byte_cursor = self.char_to_byte(self.cursor);
        let mut items: Vec<String> = Vec::new();
        let _ = completer.complete(&self.text, byte_cursor, &mut |c| {
            items.push(c.text().to_string());
            Ok(items.len() < 20)
        });
        match items.len() {
            0 => self.hide_sidekick(),
            1 => {
                self.text = items.into_iter().next().unwrap_or_default();
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
        let mut items: Vec<String> = Vec::new();
        let _ = completer.complete(&self.text, byte_cursor, &mut |c| {
            items.push(c.text().to_string());
            Ok(items.len() < 20)
        });
        if items.len() > 1 {
            self.show_sidekick(items);
        } else {
            self.hide_sidekick();
        }
    }

    fn show_sidekick(&mut self, items: Vec<String>) {
        if let Ok(mut sk) = self.sidekick.lock() {
            sk.set_items(items, 0);
        }
        if !self.sidekick_visible {
            self.sidekick_visible = true;
            self.emit_sidekick_show();
        }
    }

    pub(crate) fn hide_sidekick(&mut self) {
        if self.sidekick_visible {
            self.sidekick_visible = false;
            self.state.put_command(CM_SIDEKICK_HIDE, None);
        }
    }

    fn emit_sidekick_show(&mut self) {
        let b = self.state.bounds();
        let h = self.sidekick.lock().map(|sk| sk.len()).unwrap_or(0).min(8) as u16;
        let rect = Rect::new(b.x, b.y.saturating_sub(h), b.w.max(20), h);
        let data = SidekickShow {
            rect,
            view: Arc::clone(&self.sidekick) as Arc<Mutex<dyn View>>,
        };
        self.state.put_command(CM_SIDEKICK_SHOW, Some(Box::new(data)));
    }

    /// Apply the currently selected sidekick item.
    pub(crate) fn apply_sidekick_selection(&mut self) {
        let text = self
            .sidekick
            .lock()
            .ok()
            .and_then(|sk| sk.selected_text().map(String::from));
        if let Some(t) = text {
            self.text = t;
            self.cursor = self.char_count();
            self.update_width();
        }
        self.hide_sidekick();
    }
}
