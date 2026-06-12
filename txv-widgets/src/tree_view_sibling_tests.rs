//! Tests for TreeData::is_last_sibling default implementation.

#[cfg(test)]
mod tests {
    use crate::tree_view::TreeData;

    /// A mock tree represented as a flat list of (id, depth) pairs.
    /// All nodes are "visible" in order.
    struct MockTree {
        nodes: Vec<(usize, usize)>, // (id, depth)
    }

    impl MockTree {
        fn new(depths: &[usize]) -> Self {
            Self {
                nodes: depths.iter().enumerate().map(|(i, &d)| (i, d)).collect(),
            }
        }
    }

    impl TreeData for MockTree {
        fn root_count(&self) -> usize {
            self.nodes.iter().filter(|(_, d)| *d == 0).count()
        }
        fn child_count(&self, _id: usize) -> usize {
            0
        }
        fn label(&self, _id: usize) -> &str {
            "node"
        }
        fn is_expandable(&self, _id: usize) -> bool {
            false
        }
        fn is_expanded(&self, _id: usize) -> bool {
            false
        }
        fn toggle(&mut self, _id: usize) {}
        fn depth(&self, id: usize) -> usize {
            self.nodes[id].1
        }
        fn visible_count(&self) -> usize {
            self.nodes.len()
        }
        fn visible_id(&self, row: usize) -> usize {
            self.nodes[row].0
        }
    }

    /// Helper: compute is_last_sibling for all rows, returns a string like "├├└├└└"
    fn sibling_pattern(depths: &[usize]) -> String {
        let tree = MockTree::new(depths);
        (0..tree.visible_count())
            .map(|row| {
                if tree.is_last_sibling(row) {
                    '└'
                } else {
                    '├'
                }
            })
            .collect()
    }

    // ──── Single level (flat list) ────

    #[test]
    fn flat_single_node() {
        // A
        assert_eq!(sibling_pattern(&[0]), "└");
    }

    #[test]
    fn flat_three_siblings() {
        // A
        // B
        // C
        assert_eq!(sibling_pattern(&[0, 0, 0]), "├├└");
    }

    // ──── Two levels ────

    #[test]
    fn parent_with_single_child() {
        // A
        //   B
        assert_eq!(sibling_pattern(&[0, 1]), "└└");
    }

    #[test]
    fn parent_with_two_children() {
        // A
        //   B
        //   C
        assert_eq!(sibling_pattern(&[0, 1, 1]), "└├└");
    }

    #[test]
    fn two_parents_each_with_children() {
        // A
        //   A1
        //   A2
        // B
        //   B1
        assert_eq!(sibling_pattern(&[0, 1, 1, 0, 1]), "├├└└└");
    }

    // ──── Three levels ────

    #[test]
    fn three_level_chain() {
        // A
        //   B
        //     C
        assert_eq!(sibling_pattern(&[0, 1, 2]), "└└└");
    }

    #[test]
    fn parent_child_with_grandchildren() {
        // A
        //   B
        //     B1
        //     B2
        //   C
        assert_eq!(sibling_pattern(&[0, 1, 2, 2, 1]), "└├├└└");
    }

    // ──── Complex tree (your todo-tree-like structure) ────

    #[test]
    fn complex_multi_level() {
        // High priority        (0, depth 0) — has sibling "New"
        //   MCP permissions    (1, depth 1) — has sibling
        //   Search & replace   (2, depth 1) — has sibling
        //   Sticky scroll      (3, depth 1) — last child of "High"
        // New                  (4, depth 0) — has sibling "Medium"
        //   remove statusbar   (5, depth 1) — has sibling
        //   take vi to txv     (6, depth 1) — has children, has sibling
        //     Create crate     (7, depth 2) — has sibling
        //     Define EditorC   (8, depth 2) — last child
        //   check duplication  (9, depth 1) — has sibling
        //   txv lint cleanup   (10, depth 1) — LAST child of "New"
        //     Fix pub-fields   (11, depth 2) — has sibling
        //     Fix deep-path    (12, depth 2) — last child
        // Medium               (13, depth 0) — last root
        //   formatter          (14, depth 1) — last child
        let depths = [0, 1, 1, 1, 0, 1, 1, 2, 2, 1, 1, 2, 2, 0, 1];
        let pattern = sibling_pattern(&depths);
        //       High  MCP  S&R  Sticky  New  rmSB  viTxv  Cr  Def  chkD  lint  pubF  dpP  Med  fmt
        assert_eq!(pattern, "├├├└├├├├└├└├└└└");
    }

    // ──── Edge cases ────

    #[test]
    fn deep_last_child_chain() {
        // A
        //   B
        //     C
        //       D
        //         E
        assert_eq!(sibling_pattern(&[0, 1, 2, 3, 4]), "└└└└└");
    }

    #[test]
    fn alternating_deep_and_shallow() {
        // A
        //   A1
        //     A1a
        //   A2
        //     A2a
        //       A2a1
        // B
        assert_eq!(sibling_pattern(&[0, 1, 2, 1, 2, 3, 0]), "├├└└└└└");
    }

    #[test]
    fn many_roots_no_children() {
        assert_eq!(sibling_pattern(&[0, 0, 0, 0, 0]), "├├├├└");
    }

    #[test]
    fn last_root_has_deep_subtree() {
        // A
        // B
        //   B1
        //     B1a
        //     B1b
        //   B2
        assert_eq!(sibling_pattern(&[0, 0, 1, 2, 2, 1]), "├└├├└└");
    }
}
