//! FileTreeData operations — load, expand, collapse, visibility.

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
