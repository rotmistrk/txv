//! FileTreeData — TreeData implementation for filesystem navigation.
//! Uses the `ignore` crate to respect .gitignore rules.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use txv_core::cell::Color;

#[derive(Clone)]
pub(crate) struct TreeNode {
    pub(crate) path: PathBuf,
    pub(crate) label: String,
    pub(crate) depth: usize,
    pub(crate) is_dir: bool,
    pub(crate) expanded: bool,
    pub(crate) parent: Option<usize>,
}

/// Filesystem tree data provider.
pub struct FileTreeData {
    root: PathBuf,
    /// Additional roots for multi-root workspace (empty = single root mode).
    pub(crate) extra_roots: Vec<PathBuf>,
    pub(crate) nodes: Vec<TreeNode>,
    pub(crate) visible: Vec<usize>,
    /// Per-path foreground color (relative path → color).
    pub(crate) colors: HashMap<String, Color>,
    /// Whether to show hidden (dot) files.
    pub show_hidden: bool,
    /// Active filter text (empty = no filter).
    pub(crate) filter: String,
    /// Indices of characters that matched in each node's label (node_id → positions).
    pub(crate) match_positions: HashMap<usize, Vec<usize>>,
    /// Per-node flag: true if any descendant matches the filter.
    pub(crate) has_match_below: Vec<bool>,
    /// Whether all directories have been recursively loaded.
    pub(crate) fully_loaded: bool,
}

impl FileTreeData {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let mut data = Self::empty(root.clone());
        data.load_children(root, None, 0);
        data.rebuild_visible();
        data
    }

    /// Create with multiple root directories. Each root becomes a top-level node.
    pub fn with_roots(roots: Vec<PathBuf>) -> Self {
        let primary = roots.first().cloned().unwrap_or_default();
        let mut data = Self::empty(primary);
        if roots.len() > 1 {
            for root_path in &roots {
                data.nodes.push(Self::root_node(root_path));
            }
            data.extra_roots = roots;
        } else if let Some(r) = roots.into_iter().next() {
            data.load_children(r, None, 0);
        }
        data.rebuild_visible();
        data
    }

    fn empty(root: PathBuf) -> Self {
        Self {
            root,
            extra_roots: Vec::new(),
            nodes: Vec::new(),
            visible: Vec::new(),
            colors: HashMap::new(),
            show_hidden: true,
            filter: String::new(),
            match_positions: HashMap::new(),
            has_match_below: Vec::new(),
            fully_loaded: false,
        }
    }

    fn root_node(path: &Path) -> TreeNode {
        let label = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        TreeNode {
            path: path.to_path_buf(),
            label,
            depth: 0,
            is_dir: true,
            expanded: false,
            parent: None,
        }
    }

    /// Whether this is a multi-root tree (roots shown as top-level nodes).
    pub fn is_multi_root(&self) -> bool {
        !self.extra_roots.is_empty()
    }

    /// All root paths (for multi-root; single-root returns just the primary).
    pub fn all_roots(&self) -> Vec<&Path> {
        if self.extra_roots.is_empty() {
            vec![self.root.as_path()]
        } else {
            self.extra_roots.iter().map(|p| p.as_path()).collect()
        }
    }

    /// Return the root directory that contains the given node.
    pub fn root_of(&self, id: usize) -> &Path {
        if self.extra_roots.is_empty() {
            return &self.root;
        }
        // Walk up to find the top-level ancestor.
        let mut current = id;
        while let Some(parent) = self.nodes[current].parent {
            current = parent;
        }
        &self.nodes[current].path
    }

    pub fn path(&self, id: usize) -> &Path {
        &self.nodes[id].path
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Set per-path foreground colors (relative path → color).
    pub fn set_colors(&mut self, colors: HashMap<String, Color>) {
        self.colors = colors;
    }

    /// Rebuild the tree from disk, preserving expanded directories.
    pub fn refresh(&mut self) {
        let expanded_paths: Vec<PathBuf> = self
            .nodes
            .iter()
            .filter(|n| n.is_dir && n.expanded)
            .map(|n| n.path.clone())
            .collect();

        self.nodes.clear();
        self.visible.clear();
        self.fully_loaded = false;

        if self.extra_roots.is_empty() {
            let root = self.root.clone();
            self.load_children(root, None, 0);
        } else {
            for root_path in &self.extra_roots.clone() {
                self.nodes.push(Self::root_node(root_path));
            }
        }

        // Re-expand previously expanded directories
        for path in &expanded_paths {
            if let Some(idx) = self.nodes.iter().position(|n| n.path == *path) {
                if self.nodes[idx].is_dir && !self.nodes[idx].expanded {
                    self.nodes[idx].expanded = true;
                    let depth = self.nodes[idx].depth;
                    self.load_children(path.clone(), Some(idx), depth + 1);
                }
            }
        }

        self.rebuild_visible();
    }

    /// Return paths of all currently expanded directories.
    pub fn expanded_paths(&self) -> Vec<PathBuf> {
        self.nodes
            .iter()
            .filter(|n| n.is_dir && n.expanded)
            .map(|n| n.path.clone())
            .collect()
    }

    /// Expand directories matching the given paths.
    pub fn expand_paths(&mut self, paths: &[PathBuf]) {
        for path in paths {
            if let Some(idx) = self.nodes.iter().position(|n| n.path == *path) {
                if self.nodes[idx].is_dir && !self.nodes[idx].expanded {
                    self.nodes[idx].expanded = true;
                    let depth = self.nodes[idx].depth;
                    let has_children = self.nodes.iter().any(|n| n.parent == Some(idx));
                    if !has_children {
                        self.load_children(path.clone(), Some(idx), depth + 1);
                    }
                }
            }
        }
        self.rebuild_visible();
    }

    pub(crate) fn load_children(&mut self, dir: PathBuf, parent: Option<usize>, depth: usize) {
        let walker = WalkBuilder::new(&dir)
            .max_depth(Some(1))
            .hidden(!self.show_hidden)
            .sort_by_file_name(|a, b| a.cmp(b))
            .build();

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in walker.flatten() {
            let path = entry.path().to_path_buf();
            if path == dir {
                continue;
            }
            let label = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let is_dir = path.is_dir();
            let node = TreeNode {
                path,
                label,
                depth,
                is_dir,
                expanded: false,
                parent,
            };
            if is_dir {
                dirs.push(node);
            } else {
                files.push(node);
            }
        }

        // Dirs first, then files
        self.nodes.extend(dirs);
        self.nodes.extend(files);
    }

    pub(crate) fn rebuild_visible(&mut self) {
        self.visible.clear();
        self.collect_visible(None, 0);
    }

    fn collect_visible(&mut self, parent: Option<usize>, depth: usize) {
        self.collect_visible_inner(parent, depth, false);
    }

    /// `ancestor_matched` = an ancestor's name matched directly → show all descendants.
    fn collect_visible_inner(&mut self, parent: Option<usize>, depth: usize, ancestor_matched: bool) {
        let ids: Vec<usize> = self
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.parent == parent && n.depth == depth)
            .map(|(i, _)| i)
            .collect();
        for id in ids {
            if !self.filter.is_empty() && !ancestor_matched && !self.node_passes_filter(id) {
                continue;
            }
            self.visible.push(id);
            if self.nodes[id].is_dir && self.nodes[id].expanded {
                let propagate = ancestor_matched || self.node_matches_directly(id);
                self.collect_visible_inner(Some(id), depth + 1, propagate);
            }
        }
    }

    pub(crate) fn expand_node(&mut self, id: usize) {
        if !self.nodes[id].is_dir || self.nodes[id].expanded {
            return;
        }
        self.nodes[id].expanded = true;
        let path = self.nodes[id].path.clone();
        let depth = self.nodes[id].depth + 1;
        // Only load if not already loaded
        let has_children = self.nodes.iter().any(|n| n.parent == Some(id));
        if !has_children {
            self.load_children(path, Some(id), depth);
        }
        self.rebuild_visible();
    }

    pub(crate) fn collapse_node(&mut self, id: usize) {
        if !self.nodes[id].expanded {
            return;
        }
        self.nodes[id].expanded = false;
        self.rebuild_visible();
    }
}
