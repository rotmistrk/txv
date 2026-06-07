//! FileTreeData operations — load, expand, collapse, visibility.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use ignore::WalkBuilder;

use crate::file_tree::{FileTreeData, TreeNode};

impl FileTreeData {
    pub(crate) fn load_children(&mut self, dir: PathBuf, parent: Option<usize>, depth: usize) {
        let walker = WalkBuilder::new(&dir)
            .max_depth(Some(1))
            .hidden(!self.show_hidden)
            .sort_by_file_name(|a, b| a.cmp(b))
            .build();

        let mut dirs = Vec::new();
        let mut files = Vec::new();
        let mut tracked_paths: HashSet<PathBuf> = HashSet::new();

        for entry in walker.flatten() {
            let path = entry.path().to_path_buf();
            if path == dir {
                continue;
            }
            tracked_paths.insert(path.clone());
            let node = Self::make_node(path, parent, depth, false);
            if node.is_dir {
                dirs.push(node);
            } else {
                files.push(node);
            }
        }

        let (ignored_dirs, ignored_files) = self.collect_ignored(&dir, &tracked_paths, parent, depth);

        self.nodes.extend(dirs);
        self.nodes.extend(files);
        self.nodes.extend(ignored_dirs);
        self.nodes.extend(ignored_files);
    }

    fn make_node(path: PathBuf, parent: Option<usize>, depth: usize, ignored: bool) -> TreeNode {
        let label = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let is_dir = path.is_dir();
        TreeNode {
            path,
            label,
            depth,
            is_dir,
            expanded: false,
            parent,
            ignored,
        }
    }

    fn collect_ignored(
        &self,
        dir: &PathBuf,
        tracked_paths: &HashSet<PathBuf>,
        parent: Option<usize>,
        depth: usize,
    ) -> (Vec<TreeNode>, Vec<TreeNode>) {
        if !self.show_ignored {
            return (Vec::new(), Vec::new());
        }
        let mut ignored_dirs = Vec::new();
        let mut ignored_files = Vec::new();
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return (ignored_dirs, ignored_files),
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if tracked_paths.contains(&path) {
                continue;
            }
            let label = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if !self.show_hidden && label.starts_with('.') {
                continue;
            }
            let node = Self::make_node(path, parent, depth, true);
            if node.is_dir {
                ignored_dirs.push(node);
            } else {
                ignored_files.push(node);
            }
        }
        ignored_dirs.sort_by(|a, b| a.label.cmp(&b.label));
        ignored_files.sort_by(|a, b| a.label.cmp(&b.label));
        (ignored_dirs, ignored_files)
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
            let node = &self.nodes[id];
            // Skip ignored nodes during filter unless they are expanded
            if !self.filter.is_empty() && node.ignored && !node.expanded {
                continue;
            }
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
