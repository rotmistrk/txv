//! InputLine completion and sidekick integration.

use txv_core::prelude::*;

use super::completion_item::CompletionItem;
use super::completion_source::CompletionSource;
use super::InputLine;
use crate::dropdown_menu::{DropdownMenu, FilterMode, NumberMode, OpenSide};
use crate::sidekick::{SidekickRequest, CM_SIDEKICK_HIDE, CM_SIDEKICK_SHOW};

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
                self.update_width();
                if self.text.ends_with('/') {
                    self.update_completions();
                } else {
                    self.hide_sidekick();
                }
            }
            _ => {
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
        if items.is_empty() {
            self.hide_sidekick();
        } else {
            self.show_sidekick(items);
        }
    }

    fn show_sidekick(&mut self, items: Vec<CompletionItem>) {
        let count = items.len();
        let max_w = items.iter().map(|i| i.display().len()).max().unwrap_or(0);
        let source = CompletionSource::new(items);
        let menu = DropdownMenu::new(source)
            .with_numbers(NumberMode::None)
            .with_filter(FilterMode::None)
            .with_open_side(OpenSide::None);
        let content_h = count.min(8) as u16;
        let h = content_h + 2;
        let w = (max_w as u16 + 4).clamp(14, 42);
        let rect = Rect::new(0, 0, w, h);
        let data = SidekickRequest::new(Box::new(menu), rect, self.state.id());
        self.state.put_command(CM_SIDEKICK_SHOW, Some(Box::new(data)));
        self.sidekick_visible = true;
    }

    pub(crate) fn hide_sidekick(&mut self) {
        if self.sidekick_visible {
            self.sidekick_visible = false;
            self.state.put_command(CM_SIDEKICK_HIDE, None);
        }
    }

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
}
