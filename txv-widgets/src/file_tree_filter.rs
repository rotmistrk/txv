//! FileTreeData filter — pure visibility mask, no structural changes.
//!
//! The filter does NOT change expand/collapse state. It only controls which
//! nodes are included in the visible list during `rebuild_visible`.

use crate::file_tree::FileTreeData;

impl FileTreeData {
    /// Set the filter text. Empty string clears the filter.
    pub fn set_filter(&mut self, text: &str) {
        self.filter = text.to_lowercase();
        self.match_positions.clear();
        self.has_match_below.clear();
        if !self.filter.is_empty() {
            self.compute_matches();
            self.compute_has_match_below();
        }
        self.rebuild_visible();
    }

    /// Pre-load all directories for filtering. Call once on filter activation.
    pub fn ensure_all_loaded(&mut self) {
        if self.fully_loaded {
            return;
        }
        use std::collections::HashSet;
        const MAX_DEPTH: usize = 10;
        let mut parents: HashSet<usize> = HashSet::new();
        for n in &self.nodes {
            if let Some(p) = n.parent {
                parents.insert(p);
            }
        }
        let mut i = 0;
        while i < self.nodes.len() {
            let dominated = self.nodes[i].is_dir
                && !parents.contains(&i)
                && self.nodes[i].depth < MAX_DEPTH
                && self.nodes[i].label != ".git";
            if dominated {
                let path = self.nodes[i].path.clone();
                let depth = self.nodes[i].depth + 1;
                let before = self.nodes.len();
                self.load_children(path, Some(i), depth);
                for j in before..self.nodes.len() {
                    if let Some(p) = self.nodes[j].parent {
                        parents.insert(p);
                    }
                }
            }
            i += 1;
        }
        self.fully_loaded = true;
    }

    /// Current filter text.
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Get match positions for a node (for highlight rendering).
    pub fn match_positions(&self, id: usize) -> Option<&[usize]> {
        self.match_positions.get(&id).map(|v| v.as_slice())
    }

    /// Does this node pass the filter? (matches directly or has matching descendants)
    pub(super) fn node_passes_filter(&self, id: usize) -> bool {
        if self.filter.is_empty() {
            return true;
        }
        self.match_positions.contains_key(&id) || self.has_match_below.get(id).copied().unwrap_or(false)
    }

    /// Does this node match the filter directly (by its own name)?
    pub(super) fn node_matches_directly(&self, id: usize) -> bool {
        self.match_positions.contains_key(&id)
    }

    /// Compute which nodes match the filter by name.
    fn compute_matches(&mut self) {
        for (id, node) in self.nodes.iter().enumerate() {
            if let Some(positions) = fuzzy_match_positions(&node.label.to_lowercase(), &self.filter) {
                self.match_positions.insert(id, positions);
            }
        }
    }

    /// Bottom-up pass: mark dirs that have matching descendants.
    fn compute_has_match_below(&mut self) {
        self.has_match_below.resize(self.nodes.len(), false);
        // Walk nodes; for each match, mark all ancestors
        for id in 0..self.nodes.len() {
            if self.match_positions.contains_key(&id) {
                let mut cur = self.nodes[id].parent;
                while let Some(p) = cur {
                    if self.has_match_below[p] {
                        break; // already marked upward
                    }
                    self.has_match_below[p] = true;
                    cur = self.nodes[p].parent;
                }
            }
        }
    }
}

/// Subsequence fuzzy match returning matched character positions.
pub fn fuzzy_match_positions(haystack: &str, needle: &str) -> Option<Vec<usize>> {
    let mut positions = Vec::new();
    let mut chars = needle.chars();
    let mut current = chars.next()?;
    for (i, h) in haystack.chars().enumerate() {
        if h == current {
            positions.push(i);
            match chars.next() {
                Some(c) => current = c,
                None => return Some(positions),
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_tree::FileTreeData;
    use crate::tree_view::TreeData;

    #[test]
    fn fuzzy_match_basic() {
        let result = fuzzy_match_positions("movement.rs", "mvt");
        assert_eq!(result, Some(vec![0, 2, 7]));
    }

    #[test]
    fn fuzzy_match_no_match() {
        let result = fuzzy_match_positions("hello", "xyz");
        assert_eq!(result, None);
    }

    #[test]
    fn fuzzy_match_exact() {
        let result = fuzzy_match_positions("mod.rs", "mod");
        assert_eq!(result, Some(vec![0, 1, 2]));
    }

    #[test]
    fn filter_hides_non_matching_files() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("main.rs"), "").unwrap();
        std::fs::write(dir.path().join("lib.rs"), "").unwrap();
        std::fs::write(dir.path().join("test.txt"), "").unwrap();

        let mut data = FileTreeData::new(dir.path());
        data.set_filter("rs");
        let visible: Vec<&str> = (0..data.visible_count())
            .map(|i| data.label(data.visible_id(i)))
            .collect();
        assert!(!visible.contains(&"test.txt"));
        assert!(visible.contains(&"main.rs"));
        assert!(visible.contains(&"lib.rs"));
    }

    #[test]
    fn clear_filter_restores_all() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("main.rs"), "").unwrap();
        std::fs::write(dir.path().join("test.txt"), "").unwrap();

        let mut data = FileTreeData::new(dir.path());
        let total = data.visible_count();
        data.set_filter("rs");
        assert!(data.visible_count() < total);
        data.set_filter("");
        assert_eq!(data.visible_count(), total);
    }

    #[test]
    fn filter_match_positions_recorded() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("movement.rs"), "").unwrap();

        let mut data = FileTreeData::new(dir.path());
        data.set_filter("mvt");
        let mut found = false;
        for i in 0..data.visible_count() {
            let id = data.visible_id(i);
            if data.label(id) == "movement.rs" {
                assert_eq!(data.match_positions(id), Some([0, 2, 7].as_slice()));
                found = true;
            }
        }
        assert!(found);
    }

    #[test]
    fn filter_shows_closed_dir_with_matches_inside() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("src");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("deep.rs"), "").unwrap();
        std::fs::write(dir.path().join("top.txt"), "").unwrap();

        let mut data = FileTreeData::new(dir.path());
        data.ensure_all_loaded();
        data.set_filter("deep");
        let vis = |d: &FileTreeData| -> Vec<String> {
            (0..d.visible_count())
                .map(|i| d.label(d.visible_id(i)).to_string())
                .collect()
        };
        // src/ visible (has match below) but collapsed — children hidden
        assert!(vis(&data).contains(&"src".to_string()));
        assert!(!vis(&data).contains(&"deep.rs".to_string()));
        // Expand src/ — now deep.rs appears
        let src_id = data.nodes.iter().position(|n| n.label == "src").unwrap();
        data.toggle(src_id);
        assert!(vis(&data).contains(&"deep.rs".to_string()));
    }

    #[test]
    fn filter_shows_children_of_expanded_dir() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("src");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("deep.rs"), "").unwrap();
        std::fs::write(sub.join("other.txt"), "").unwrap();

        let mut data = FileTreeData::new(dir.path());
        data.ensure_all_loaded();
        let src_id = data.nodes.iter().position(|n| n.label == "src").unwrap();
        data.toggle(src_id);
        data.set_filter("deep");
        let visible: Vec<&str> = (0..data.visible_count())
            .map(|i| data.label(data.visible_id(i)))
            .collect();
        assert!(visible.contains(&"src") && visible.contains(&"deep.rs"));
        assert!(!visible.contains(&"other.txt"));
    }

    #[test]
    fn dir_name_match_shows_all_children() {
        let dir = tempfile::tempdir().unwrap();
        let hooks = dir.path().join("hooks");
        std::fs::create_dir(&hooks).unwrap();
        std::fs::write(hooks.join("pre-commit"), "").unwrap();
        std::fs::write(hooks.join("post-merge"), "").unwrap();

        let mut data = FileTreeData::new(dir.path());
        data.ensure_all_loaded();
        let hooks_id = data.nodes.iter().position(|n| n.label == "hooks").unwrap();
        data.toggle(hooks_id);
        data.set_filter("hooks");
        let visible: Vec<&str> = (0..data.visible_count())
            .map(|i| data.label(data.visible_id(i)))
            .collect();
        assert!(visible.contains(&"hooks") && visible.contains(&"pre-commit"));
        assert!(visible.contains(&"post-merge"));
    }

    #[test]
    fn collapse_during_filter_hides_children_keeps_dir() {
        let dir = tempfile::tempdir().unwrap();
        let doc = dir.path().join("doc");
        std::fs::create_dir(&doc).unwrap();
        std::fs::write(doc.join("readme.md"), "").unwrap();

        let mut data = FileTreeData::new(dir.path());
        data.ensure_all_loaded();
        let doc_id = data.nodes.iter().position(|n| n.label == "doc").unwrap();
        data.toggle(doc_id);
        data.set_filter("md");
        let vis = |d: &FileTreeData| -> Vec<String> {
            (0..d.visible_count())
                .map(|i| d.label(d.visible_id(i)).to_string())
                .collect()
        };
        assert!(vis(&data).contains(&"readme.md".to_string()));
        data.toggle(doc_id); // collapse
        assert!(vis(&data).contains(&"doc".to_string()), "dir stays visible");
        assert!(!vis(&data).contains(&"readme.md".to_string()), "children hidden");
    }
}
