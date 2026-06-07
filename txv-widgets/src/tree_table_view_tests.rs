//! Tests for TreeTableView — navigation, expand/collapse, rendering, scroll.

use txv_core::prelude::*;

use crate::tree_table_source::TreeTableSource;
use crate::tree_table_view::TreeTableView;

/// Test data: 3 roots, second is expandable with 2 children, 1 extra column.
struct TestSource {
    expanded: bool,
}

impl TestSource {
    fn new() -> Self {
        Self { expanded: false }
    }

    fn rows(&self) -> Vec<(&str, usize, bool)> {
        // (label, depth, expandable)
        let mut rows = vec![("Alpha", 0, false), ("Beta", 0, true)];
        if self.expanded {
            rows.push(("Child1", 1, false));
            rows.push(("Child2", 1, false));
        }
        rows.push(("Gamma", 0, false));
        rows
    }
}

impl TreeTableSource for TestSource {
    fn visible_count(&self) -> usize {
        self.rows().len()
    }

    fn label(&self, row: usize) -> &str {
        match self.rows().get(row) {
            Some((l, _, _)) => l,
            None => "",
        }
    }

    fn depth(&self, row: usize) -> usize {
        self.rows().get(row).map_or(0, |r| r.1)
    }

    fn is_expandable(&self, row: usize) -> bool {
        self.rows().get(row).map_or(false, |r| r.2)
    }

    fn is_expanded(&self, _row: usize) -> bool {
        self.expanded
    }

    fn toggle(&mut self, row: usize) {
        if self.is_expandable(row) {
            self.expanded = !self.expanded;
        }
    }

    fn column_count(&self) -> usize {
        1
    }

    fn cell(&self, row: usize, _col: usize) -> &str {
        match row {
            0 => "tag-a",
            1 => "tag-b",
            _ => "",
        }
    }
}

fn make_view() -> TreeTableView<TestSource> {
    let mut v = TreeTableView::new(TestSource::new(), &[6]);
    v.state.set_bounds(Rect::new(0, 0, 40, 10));
    v.select();
    v
}

fn row_text(v: &TreeTableView<TestSource>, y: u16) -> String {
    let buf = v.buffer();
    let mut s = String::new();
    for x in 0..buf.width() {
        s.push(buf.cell(x, y).ch());
    }
    s.trim_end().to_string()
}

#[test]
fn draw_renders_labels_and_columns() {
    let mut v = make_view();
    v.draw();
    let r0 = row_text(&v, 0);
    let r1 = row_text(&v, 1);
    assert!(r0.contains("Alpha"), "row 0: {r0}");
    assert!(r0.contains("tag-a"), "row 0 col: {r0}");
    assert!(r1.contains("Beta"), "row 1: {r1}");
    assert!(r1.contains("tag-b"), "row 1 col: {r1}");
}

#[test]
fn navigate_down_moves_cursor() {
    let mut v = make_view();
    assert_eq!(v.cursor(), 0);
    let down = Event::Key(KeyEvent::new(KeyCode::Down, KeyMod::default()));
    v.handle(&down);
    assert_eq!(v.cursor(), 1);
    v.handle(&down);
    assert_eq!(v.cursor(), 2);
}

#[test]
fn navigate_up_moves_cursor() {
    let mut v = make_view();
    v.set_cursor(2);
    let up = Event::Key(KeyEvent::new(KeyCode::Up, KeyMod::default()));
    v.handle(&up);
    assert_eq!(v.cursor(), 1);
}

#[test]
fn expand_on_right_key() {
    let mut v = make_view();
    v.set_cursor(1); // Beta (expandable)
    let right = Event::Key(KeyEvent::new(KeyCode::Right, KeyMod::default()));
    v.handle(&right);
    assert!(v.data().expanded);
    assert_eq!(v.data().visible_count(), 5);
}

#[test]
fn collapse_on_left_key() {
    let mut v = make_view();
    v.set_cursor(1);
    v.data_mut().expanded = true;
    let left = Event::Key(KeyEvent::new(KeyCode::Left, KeyMod::default()));
    v.handle(&left);
    assert!(!v.data().expanded);
    assert_eq!(v.data().visible_count(), 3);
}

#[test]
fn left_on_child_goes_to_parent() {
    let mut v = make_view();
    v.data_mut().expanded = true;
    v.set_cursor(2); // Child1, depth=1
    let left = Event::Key(KeyEvent::new(KeyCode::Left, KeyMod::default()));
    v.handle(&left);
    assert_eq!(v.cursor(), 1); // Beta (parent)
}

#[test]
fn scroll_when_content_exceeds_viewport() {
    // Viewport of 3 rows, 5 items (expanded)
    let mut v = TreeTableView::new(TestSource::new(), &[6]);
    v.state.set_bounds(Rect::new(0, 0, 40, 3));
    v.select();
    v.data_mut().expanded = true; // 5 items
    v.set_cursor(4); // last item
    assert!(v.scroll.offset > 0, "should scroll: offset={}", v.scroll.offset);
}

#[test]
fn extra_column_at_correct_position() {
    let mut v = make_view();
    v.draw();
    // col_widths=[6], so extra_total = 6+1 = 7, tree_w = 40-7 = 33
    // Separator at x=33, cell starts at x=34
    let buf = v.buffer();
    let sep = buf.cell(33, 0);
    assert_eq!(sep.ch(), '\u{2502}', "separator at x=33");
    let c = buf.cell(34, 0);
    assert_eq!(c.ch(), 't', "cell content starts at x=34");
}

// === Regression: collapse must clear stale rows below visible items (61ee5aa) ===

#[test]
fn collapse_clears_stale_rows_below_content() {
    let mut v = TreeTableView::new(TestSource::new(), &[6]);
    v.state.set_bounds(Rect::new(0, 0, 40, 10));
    v.select();

    // Expand Beta to show children (5 items total)
    v.data_mut().expanded = true;
    v.draw();

    // Verify Child1 is drawn on row 2
    let r2_before = row_text(&v, 2);
    assert!(r2_before.contains("Child1"), "Child1 on row 2 before collapse");

    // Collapse Beta (back to 3 items)
    v.set_cursor(1);
    let left = Event::Key(KeyEvent::new(KeyCode::Left, KeyMod::default()));
    v.handle(&left);
    v.draw();

    // Row 3 and 4 were previously occupied by children — they should now be cleared
    let r3 = row_text(&v, 3);
    let r4 = row_text(&v, 4);
    assert!(
        !r3.contains("Child"),
        "stale content should be cleared after collapse: row3='{r3}'"
    );
    assert!(
        !r4.contains("Child"),
        "stale content should be cleared after collapse: row4='{r4}'"
    );
}
