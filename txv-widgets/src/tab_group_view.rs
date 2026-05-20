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
        let arrow_style = pal.chrome.tab_focused_arrow.to_style();
        self.group.buffer_mut().hline(0, 0, w, g.ui.separator_h, dim);
        let mut x = 0u16;
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
                if x + left_len + label_len + right_len > w {
                    break;
                }
                self.group.buffer_mut().print(x, 0, left, arrow_style);
                x += left_len;
                self.group.buffer_mut().print(x, 0, &label, focused_style);
                x += label_len;
                self.group.buffer_mut().print(x, 0, right, arrow_style);
                x += right_len;
            } else {
                let label = format!(" {title} ");
                let len = label.len() as u16;
                if x + len > w {
                    break;
                }
                self.group.buffer_mut().print(x, 0, &label, dim);
                x += len;
            }
        }
        if self.titles.len() > 1 {
            let count = format!("❨{}❩", self.titles.len());
            let clen = count.chars().count() as u16;
            if x + clen < w {
                self.group.buffer_mut().print(x + 1, 0, &count, dim);
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
