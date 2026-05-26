//! BranchView — git branch indicator as a proper View.

use std::path::{Path, PathBuf};
use std::time::Instant;

use txv_core::prelude::*;

/// A View-based status bar item that displays the current git branch.
pub struct BranchView {
    state: ViewState,
    root_dir: PathBuf,
    label_text: String,
    last_check: Instant,
    tick_count: u16,
}

impl BranchView {
    pub fn new(root_dir: PathBuf) -> Self {
        let mut view = Self {
            state: ViewState::new(ViewOptions {
                preprocess: true,
                focusable: false,
                ..ViewOptions::default()
            }),
            root_dir,
            label_text: String::new(),
            last_check: Instant::now(),
            tick_count: 0,
        };
        view.refresh();
        view
    }

    fn refresh(&mut self) {
        self.label_text = Self::read_branch(&self.root_dir).unwrap_or_default();
        self.last_check = Instant::now();
        self.update_bounds();
        self.state.mark_dirty();
    }

    fn update_bounds(&mut self) {
        let w = if self.label_text.is_empty() {
            0
        } else {
            self.label_text.len() as u16 + 2
        };
        let bounds = self.state.bounds();
        if bounds.w != w {
            self.state.set_bounds(Rect {
                x: bounds.x,
                y: bounds.y,
                w,
                h: 1,
            });
        }
    }

    fn read_branch(root: &Path) -> Option<String> {
        let head = std::fs::read_to_string(root.join(".git/HEAD")).ok()?;
        let head = head.trim();
        if let Some(r) = head.strip_prefix("ref: refs/heads/") {
            Some(format!("\u{e0a0} {r}"))
        } else if head.len() >= 7 {
            Some(format!("\u{e0a0} {}", &head[..7]))
        } else {
            None
        }
    }
}

impl View for BranchView {
    delegate_view_state!(state);

    fn draw(&mut self) {
        if self.label_text.is_empty() {
            self.state.mark_redrawn();
            return;
        }
        let style = Style {
            attrs: Attrs {
                reverse: true,
                ..Attrs::default()
            },
            ..Style::default()
        };
        let buf = self.state.buffer_mut();
        buf.fill(' ', style);
        buf.print(1, 0, &self.label_text, style);
        self.state.mark_redrawn();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Tick = event {
            self.tick_count += 1;
            if self.tick_count >= 60 {
                self.tick_count = 0;
                self.refresh();
            }
        }
        HandleResult::Ignored
    }
}
