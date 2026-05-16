//! TabBar — horizontal tab strip.

use txv_core::prelude::*;

pub struct Tab {
    pub label: String,
    pub command: CommandId,
}

pub struct TabBar {
    state: ViewState,
    pub tabs: Vec<Tab>,
    pub active: usize,
}

impl TabBar {
    pub fn new() -> Self {
        Self {
            state: ViewState::default(),
            tabs: Vec::new(),
            active: 0,
        }
    }

    pub fn add_tab(&mut self, label: impl Into<String>, command: CommandId) {
        self.tabs.push(Tab {
            label: label.into(),
            command,
        });
        self.state.mark_dirty();
    }

    pub fn remove_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.tabs.remove(index);
            if self.active >= self.tabs.len() && self.active > 0 {
                self.active -= 1;
            }
            self.state.mark_dirty();
        }
    }

    pub fn set_active(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active = index;
            self.state.mark_dirty();
        }
    }
}

impl Default for TabBar {
    fn default() -> Self {
        Self::new()
    }
}

impl View for TabBar {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let w = self.state.buf.width();
        let h = self.state.buf.height();
        if w == 0 || h == 0 {
            return;
        }
        let normal = Style::default();
        let active_style = txv_core::palette::palette().chrome.tab_focused.to_style();
        self.state.buf.hline(0, 0, w, ' ', normal);
        let mut x = 0u16;
        for (i, tab) in self.tabs.iter().enumerate() {
            let style = if i == self.active {
                active_style
            } else {
                normal
            };
            let label = format!(" {} ", tab.label);
            let len = label.len() as u16;
            if x + len > w {
                break;
            }
            self.state.buf.print(x, 0, &label, style);
            x += len;
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        match key.code {
            KeyCode::Left => {
                if self.active > 0 {
                    self.active -= 1;
                    self.state.mark_dirty();
                    self.state.put_command(self.tabs[self.active].command, None);
                }
                HandleResult::Consumed
            }
            KeyCode::Right => {
                if self.active + 1 < self.tabs.len() {
                    self.active += 1;
                    self.state.mark_dirty();
                    self.state.put_command(self.tabs[self.active].command, None);
                }
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}
