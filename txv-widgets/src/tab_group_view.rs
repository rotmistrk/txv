//! View trait implementation and chrome drawing for TabGroup.

use txv_core::prelude::*;

use super::tab_group::TabGroup;

impl TabGroup {
    pub(crate) fn draw_chrome(&mut self) {
        let w = self.group.buffer_mut().width();
        let h = self.group.buffer_mut().height();
        if w == 0 || h == 0 || self.titles.is_empty() {
            return;
        }
        let pal = palette();
        let g = glyphs();
        let dim = pal.base.dim.to_style();
        let focused_style = pal.chrome.tab_focused.to_style();
        let sep = g.ui.separator_v;

        self.group.buffer_mut().hline(0, 0, w, ' ', dim);

        let active_idx = self.group.focused_index();
        let mut x = 0u16;
        let mut rendered = 0usize;

        for (i, title) in self.titles.iter().enumerate() {
            // Separator before each tab
            if x >= w {
                break;
            }
            self.group.buffer_mut().put(x, 0, sep, dim);
            x += 1;

            // Build label: title + dirty indicator
            let dirty_mark = if self.dirty.get(i).copied().unwrap_or(false) {
                " •"
            } else {
                ""
            };
            let label = format!("{title}{dirty_mark}");
            let label_len = label.chars().count() as u16;

            // Check if this tab fits (need space for label + trailing separator)
            let needed = label_len + 1; // +1 for trailing sep or overflow
            if x + needed > w {
                // Overflow: show indicator for remaining tabs
                let hidden = self.titles.len() - i;
                let badge = format!("…{hidden}");
                let badge_len = badge.chars().count() as u16;
                if x + badge_len <= w {
                    self.group.buffer_mut().print(x, 0, &badge, dim);
                }
                break;
            }

            let style = if i == active_idx {
                focused_style
            } else {
                dim
            };
            self.group.buffer_mut().print(x, 0, &label, style);
            x += label_len;
            rendered += 1;
        }

        // Trailing separator
        if x < w && rendered > 0 {
            self.group.buffer_mut().put(x, 0, sep, dim);
        }
    }
}

/// Naming and utility methods.
impl TabGroup {
    pub fn has_tab_starting_with(&self, prefix: &str) -> bool {
        self.titles.iter().any(|t| t.starts_with(prefix))
    }

    pub fn rename_active(&mut self, new_title: impl Into<String>) {
        if let Some(title) = self.titles.get_mut(self.group.focused_index()) {
            *title = new_title.into();
            self.group.mark_dirty();
        }
    }

    /// Generate next available name like "Shell:0", "Shell:1", etc.
    pub fn next_tab_name(&self, prefix: &str) -> String {
        for n in 0..10 {
            let candidate = format!("{prefix}:{n}");
            if !self.has_tab_starting_with(&candidate) {
                return candidate;
            }
        }
        format!("{prefix}:0")
    }

    /// Rename active tab, keeping the "prefix:" part and replacing the user part.
    pub fn rename_user_part(&mut self, new_user_part: &str) {
        if let Some(title) = self.titles.get(self.group.focused_index()).cloned() {
            if let Some(colon) = title.find(':') {
                let prefix = &title[..=colon];
                self.rename_active(format!("{prefix}{new_user_part}"));
            }
        }
    }
}

impl View for TabGroup {
    delegate_group_state!(group, override { set_bounds, draw, handle });

    fn set_bounds(&mut self, r: Rect) {
        self.group.set_bounds(r);
        self.group.mark_dirty();
        let content = self.content_rect();
        if let Some(child) = self.group.focused_child_mut() {
            child.set_bounds(content);
        }
    }

    fn draw(&mut self) {
        self.group.buffer_mut().fill(' ', Style::default());
        self.draw_chrome();
        let my_bounds = self.group.bounds();
        // Draw and blit focused child
        let fi = self.group.focused_index();
        if let Some(child) = self.group.child_mut(fi) {
            child.draw();
        }
        // Blit child buffer into own buffer.
        // Safety: children and view.buf are disjoint fields of GroupState.
        let buf_ptr = self.group.buffer_mut() as *mut Buffer;
        if let Some(child) = self.group.child(fi) {
            let cb = child.bounds();
            let dx = cb.x.saturating_sub(my_bounds.x);
            let dy = cb.y.saturating_sub(my_bounds.y);
            unsafe { (*buf_ptr).blit(child.buffer(), dx, dy) };
        }
        self.draw_dropdown();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // Tick goes to ALL tabs (background tabs need it for refresh/polling)
        if matches!(event, Event::Tick) {
            for i in 0..self.group.child_count() {
                if let Some(child) = self.group.child_mut(i) {
                    child.handle(event);
                }
            }
            // Sync active tab title: append view's subtitle (e.g. OSC title)
            if let Some(child) = self.group.child(self.group.focused_index()) {
                let sub = child.subtitle();
                if let Some(stored) = self.titles.get_mut(self.group.focused_index()) {
                    // Strip any previous subtitle (after first space following ':')
                    let base = stored
                        .find(':')
                        .and_then(|c| stored[c..].find(' ').map(|s| c + s))
                        .map(|pos| &stored[..pos])
                        .unwrap_or(stored.as_str())
                        .to_string();
                    let new_title = if sub.is_empty() {
                        base
                    } else {
                        format!("{base} {sub}")
                    };
                    if *stored != new_title {
                        *stored = new_title;
                        self.group.mark_dirty();
                    }
                }
            }
            return HandleResult::Ignored;
        }
        // Dropdown intercepts all keys when open
        if self.dropdown_open() {
            if let Event::Key(key) = event {
                return self.handle_dropdown_key(key);
            }
        }
        // Alt+digit selects tab by index
        if let Event::Key(key) = event {
            if key.modifiers.alt && !key.modifiers.ctrl {
                if let KeyCode::Char(ch) = key.code {
                    if let Some(n) = ch.to_digit(10) {
                        if (n as usize) < self.group.child_count() {
                            self.set_active(n as usize);
                        }
                        return HandleResult::Consumed;
                    }
                }
            }
        }
        // All other events go to active tab only
        self.group.dispatch(event)
    }
}
