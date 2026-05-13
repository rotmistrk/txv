//! View trait implementation and chrome drawing for TabGroup.

use txv_core::prelude::*;

use super::tab_group::TabGroup;

impl TabGroup {
    pub(crate) fn draw_chrome(&self, surface: &mut Surface) {
        let b = self.group.view.bounds();
        if b.w == 0 || b.h == 0 || self.titles.is_empty() {
            return;
        }
        let pal = palette();
        let g = glyphs();
        let dim = pal.base.dim.to_style();
        let focused_style = pal.chrome.tab_focused.to_style();
        let arrow_style = pal.chrome.tab_focused_arrow.to_style();
        surface.hline(b.x, b.y, b.w, g.ui.separator_h, dim);
        let mut x = b.x;
        let active_idx = self.group.focused_index();
        for (i, title) in self.titles.iter().enumerate() {
            if i == active_idx {
                // Active tab with chrome glyphs
                let left = g.chrome.tab_left;
                let right = g.chrome.tab_right;
                let left_len = left.chars().count() as u16;
                let right_len = right.chars().count() as u16;
                let label = format!(" {title} ");
                let label_len = label.len() as u16;
                if x + left_len + label_len + right_len > b.x + b.w {
                    break;
                }
                surface.print(x, b.y, left, arrow_style);
                x += left_len;
                surface.print(x, b.y, &label, focused_style);
                x += label_len;
                surface.print(x, b.y, right, arrow_style);
                x += right_len;
            } else {
                let label = format!(" {title} ");
                let len = label.len() as u16;
                if x + len > b.x + b.w {
                    break;
                }
                surface.print(x, b.y, &label, dim);
                x += len;
            }
        }
        if self.titles.len() > 1 {
            let count = format!("❨{}❩", self.titles.len());
            let clen = count.chars().count() as u16;
            if x + clen < b.x + b.w {
                surface.print(x + 1, b.y, &count, dim);
            }
        }
    }
}

impl View for TabGroup {
    delegate_group_state!(group, override { set_bounds, draw, handle });

    fn set_bounds(&mut self, r: Rect) {
        self.group.view.set_bounds(r);
        self.group.view.mark_dirty();
        let content = self.content_rect();
        if let Some(child) = self.group.focused_child_mut() {
            child.set_bounds(content);
        }
    }

    fn draw(&self, surface: &mut Surface) {
        self.draw_chrome(surface);
        if let Some(child) = self.group.child(self.group.focused_index()) {
            child.draw(surface);
        }
        self.draw_dropdown(surface);
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        // Tick goes to ALL tabs (background tabs need it for refresh/polling)
        if matches!(event, Event::Tick) {
            for i in 0..self.group.child_count() {
                if let Some(child) = self.group.child_mut(i) {
                    child.handle(event, queue);
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
                        self.group.view.mark_dirty();
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
        self.group.dispatch(event, queue)
    }
}
