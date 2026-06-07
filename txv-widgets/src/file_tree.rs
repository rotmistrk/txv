//! FileTreeData — TreeData implementation for filesystem navigation.
//! Uses the `ignore` crate to respect .gitignore rules.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use txv_core::cell::Color;

#[derive(Clone)]
pub(crate) struct TreeNode {
    pub(crate) path: PathBuf,
    pub(crate) label: String,
    pub(crate) depth: usize,
    pub(crate) is_dir: bool,
    pub(crate) expanded: bool,
    pub(crate) parent: Option<usize>,
    pub(crate) ignored: bool,
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
    /// Badge colors for root nodes (index matches root order in extra_roots).
    pub(crate) root_badge_colors: Vec<Color>,
    /// Set of absolute paths currently open in editor tabs.
    pub(crate) open_files: std::collections::HashSet<PathBuf>,
    /// Whether to show hidden (dot) files.
    pub(crate) show_hidden: bool,
    /// Whether to show .gitignored files (dim, lazy-loaded).
    pub(crate) show_ignored: bool,
    /// Whether to show file/dir icons (Nerd Font).
    pub show_icons: bool,
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
            // Auto-expand all roots on initial construction
            for i in 0..data.nodes.len() {
                data.nodes[i].expanded = true;
                let path = data.nodes[i].path.clone();
                data.load_children(path, Some(i), 1);
            }
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
            root_badge_colors: Vec::new(),
            open_files: std::collections::HashSet::new(),
            show_hidden: true,
            show_ignored: true,
            show_icons: false,
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
            ignored: false,
        }
    }

    /// Whether this is a multi-root tree (roots shown as top-level nodes).
    pub fn is_multi_root(&self) -> bool {
        !self.extra_roots.is_empty()
    }

    /// Replace the set of roots and rebuild the tree.
    pub fn set_roots(&mut self, roots: Vec<PathBuf>) {
        if roots.is_empty() {
            return;
        }
        self.nodes.clear();
        self.visible.clear();
        self.fully_loaded = false;
        if roots.len() > 1 {
            self.root = roots.first().cloned().unwrap_or_default();
            self.extra_roots = roots;
            for root_path in &self.extra_roots.clone() {
                self.nodes.push(Self::root_node(root_path));
            }
        } else if let Some(r) = roots.into_iter().next() {
            self.root = r.clone();
            self.extra_roots.clear();
            self.load_children(r, None, 0);
        }
        self.rebuild_visible();
    }

    /// Set disambiguated display labels for root nodes (multi-root only).
    pub fn set_root_labels(&mut self, labels: &[String]) {
        if self.extra_roots.is_empty() {
            return;
        }
        for (i, node) in self.nodes.iter_mut().filter(|n| n.parent.is_none()).enumerate() {
            if let Some(label) = labels.get(i) {
                node.label = label.clone();
            }
        }
    }

    /// Update the set of currently open file paths.
    pub fn set_open_files(&mut self, paths: std::collections::HashSet<PathBuf>) {
        self.open_files = paths;
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

    /// Set badge colors for root nodes (one per root, in order).
    pub fn set_root_badge_colors(&mut self, colors: Vec<Color>) {
        self.root_badge_colors = colors;
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
            // Expand root nodes that were previously expanded
            for i in 0..self.nodes.len() {
                if self.nodes[i].parent.is_none()
                    && self.nodes[i].is_dir
                    && expanded_paths.contains(&self.nodes[i].path)
                {
                    self.nodes[i].expanded = true;
                    let depth = self.nodes[i].depth;
                    let path = self.nodes[i].path.clone();
                    self.load_children(path, Some(i), depth + 1);
                }
            }
        }

        // Re-expand previously expanded subdirectories
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
}
