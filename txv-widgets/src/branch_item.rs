//! BranchItem — displays the current git branch name.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use txv_core::status::{Gravity, VisibleItem};

/// Displays the current git branch name read from .git/HEAD.
pub struct BranchItem {
    root_dir: PathBuf,
    label_text: String,
    last_check: Instant,
}

impl BranchItem {
    pub fn new(root_dir: PathBuf) -> Self {
        let mut item = Self {
            root_dir,
            label_text: String::new(),
            last_check: Instant::now(),
        };
        item.refresh();
        item
    }

    fn refresh(&mut self) {
        self.label_text = Self::read_branch(&self.root_dir).unwrap_or_default();
        self.last_check = Instant::now();
    }

    fn read_branch(root: &Path) -> Option<String> {
        let head = fs::read_to_string(root.join(".git/HEAD")).ok()?;
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

impl VisibleItem for BranchItem {
    fn label(&self) -> &str {
        &self.label_text
    }
    fn gravity(&self) -> Gravity {
        Gravity::Right
    }
    fn tick(&mut self) {
        if self.last_check.elapsed().as_secs() >= 30 {
            self.refresh();
        }
    }
}
