//! TreeNode — a single node in the file tree.

use std::path::PathBuf;

/// A node in the file tree (file or directory).
pub(crate) struct TreeNode {
    pub(crate) path: PathBuf,
    pub(crate) label: String,
    pub(crate) depth: usize,
    pub(crate) is_dir: bool,
    pub(crate) expanded: bool,
    pub(crate) parent: Option<usize>,
    pub(crate) ignored: bool,
}
