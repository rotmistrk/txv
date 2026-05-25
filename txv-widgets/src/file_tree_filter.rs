//! FileTreeData filter support — fuzzy matching and visibility computation.

use crate::file_tree::FileTreeData;

impl FileTreeData {
    /// Set the filter text. Empty string clears the filter.
    pub fn set_filter(&mut self, text: &str) {
        self.filter = text.to_lowercase();
        self.match_positions.clear();
        if !self.filter.is_empty() {
            self.ensure_all_loaded();
            self.compute_matches();
        }
        self.rebuild_visible();
    }

    /// Current filter text.
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Ensure all directories have their children loaded (for full-tree search).
    fn ensure_all_loaded(&mut self) {
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
            if self.nodes[i].is_dir && !parents.contains(&i) && self.nodes[i].depth < MAX_DEPTH {
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

    /// Get match positions for a node (for highlight rendering).
    pub fn match_positions(&self, id: usize) -> Option<&[usize]> {
        self.match_positions.get(&id).map(|v| v.as_slice())
    }

    /// Compute which nodes match the filter and record char positions.
    pub(super) fn compute_matches(&mut self) {
        for (id, node) in self.nodes.iter().enumerate() {
            if node.is_dir {
                continue;
            }
            if let Some(positions) = fuzzy_match_positions(&node.label.to_lowercase(), &self.filter) {
                self.match_positions.insert(id, positions);
            }
        }
    }

    /// Check if a node (or any descendant) matches the filter.
    pub(super) fn node_matches_filter(&self, id: usize) -> bool {
        if self.filter.is_empty() {
            return true;
        }
        if self.match_positions.contains_key(&id) {
            return true;
        }
        if self.nodes[id].is_dir {
            return self
                .nodes
                .iter()
                .enumerate()
                .any(|(i, n)| n.parent == Some(id) && self.node_matches_filter(i));
        }
        false
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
        let total = data.visible_count();

        data.set_filter("rs");
        assert!(data.visible_count() < total, "filter should reduce visible count");
        let visible_labels: Vec<&str> = (0..data.visible_count())
            .map(|i| data.label(data.visible_id(i)))
            .collect();
        assert!(!visible_labels.contains(&"test.txt"), "test.txt should be hidden");
        assert!(visible_labels.contains(&"main.rs"));
        assert!(visible_labels.contains(&"lib.rs"));
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

        // Find the node for movement.rs
        let count = data.visible_count();
        let mut found = false;
        for i in 0..count {
            let id = data.visible_id(i);
            if data.label(id) == "movement.rs" {
                let positions = data.match_positions(id);
                assert_eq!(positions, Some([0, 2, 7].as_slice()));
                found = true;
            }
        }
        assert!(found, "movement.rs should be visible");
    }

    #[test]
    fn filter_searches_inside_closed_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("src");
        std::fs::create_dir(&sub).unwrap();
        std::fs::write(sub.join("deep.rs"), "").unwrap();
        std::fs::write(dir.path().join("top.txt"), "").unwrap();

        let mut data = FileTreeData::new(dir.path());
        // src/ is NOT expanded — children not loaded yet
        assert!(!data.nodes.iter().any(|n| n.label == "deep.rs"));

        // Filter should find deep.rs inside closed src/
        data.set_filter("deep");
        let visible_labels: Vec<&str> = (0..data.visible_count())
            .map(|i| data.label(data.visible_id(i)))
            .collect();
        assert!(
            visible_labels.contains(&"deep.rs"),
            "should find file inside closed dir"
        );
        assert!(visible_labels.contains(&"src"), "parent dir should be visible");
        assert!(
            !visible_labels.contains(&"top.txt"),
            "non-matching file should be hidden"
        );
    }
}
