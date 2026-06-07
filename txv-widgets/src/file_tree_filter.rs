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
            if self.should_expand_for_filter(i, &parents, MAX_DEPTH) {
                self.expand_for_filter(i, &mut parents);
            }
            i += 1;
        }
        self.fully_loaded = true;
    }

    fn expand_for_filter(&mut self, i: usize, parents: &mut std::collections::HashSet<usize>) {
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

    fn should_expand_for_filter(&self, i: usize, parents: &std::collections::HashSet<usize>, max_depth: usize) -> bool {
        self.nodes[i].is_dir
            && !parents.contains(&i)
            && self.nodes[i].depth < max_depth
            && self.nodes[i].label != ".git"
            && !self.nodes[i].ignored
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
    pub(crate) fn node_passes_filter(&self, id: usize) -> bool {
        self.filter.is_empty()
            || self.match_positions.contains_key(&id)
            || self.has_match_below.get(id).copied().unwrap_or(false)
    }

    /// Does this node match the filter directly (by its own name)?
    pub(crate) fn node_matches_directly(&self, id: usize) -> bool {
        self.match_positions.contains_key(&id)
    }

    /// Compute which nodes match the filter by name.
    fn compute_matches(&mut self) {
        for (id, node) in self.nodes.iter().enumerate() {
            if node.ignored && !node.expanded {
                continue;
            }
            if let Some(positions) = fuzzy_match_positions(&node.label.to_lowercase(), &self.filter) {
                self.match_positions.insert(id, positions);
            }
        }
    }

    /// Bottom-up pass: mark dirs that have matching descendants.
    fn compute_has_match_below(&mut self) {
        self.has_match_below.resize(self.nodes.len(), false);
        for id in 0..self.nodes.len() {
            if !self.match_positions.contains_key(&id) {
                continue;
            }
            let mut cur = self.nodes[id].parent;
            while let Some(p) = cur {
                if self.has_match_below[p] {
                    break;
                }
                self.has_match_below[p] = true;
                cur = self.nodes[p].parent;
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
#[path = "file_tree_filter_tests.rs"]
mod tests;
