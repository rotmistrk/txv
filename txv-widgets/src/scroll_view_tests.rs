//! ScrollView paging: the viewport moves a page, the cursor keeps its screen row.

use super::*;

/// 300 items in a 26-row viewport: page size 25.
fn view(offset: usize) -> ScrollView {
    let mut s = ScrollView::new();
    s.set_viewport(26);
    s.set_total(300);
    s.scroll_to(offset);
    s
}

#[test]
fn page_down_from_top_moves_viewport_a_full_page() {
    let mut s = view(0);
    let cursor = s.page_down(0);
    assert_eq!(s.offset(), 25);
    assert_eq!(cursor, 25);
}

#[test]
fn page_down_keeps_cursor_screen_row() {
    let mut s = view(50);
    let cursor = s.page_down(60);
    assert_eq!(cursor - s.offset(), 60 - 50);
}

#[test]
fn page_down_near_end_clamps_viewport_and_cursor() {
    let mut s = view(270);
    let cursor = s.page_down(290);
    assert_eq!(s.offset(), 300 - 26);
    assert_eq!(cursor, 299);
}

#[test]
fn page_up_moves_viewport_back_a_page() {
    let mut s = view(100);
    let cursor = s.page_up(110);
    assert_eq!(s.offset(), 75);
    assert_eq!(cursor, 85);
}

#[test]
fn page_up_near_top_clamps_to_zero() {
    let mut s = view(10);
    let cursor = s.page_up(12);
    assert_eq!(s.offset(), 0);
    assert_eq!(cursor, 0);
}

#[test]
fn down_then_up_returns_to_start() {
    let mut s = view(0);
    let c = s.page_down(3);
    let c = s.page_down(c);
    let c = s.page_up(c);
    let c = s.page_up(c);
    assert_eq!((s.offset(), c), (0, 3));
}

#[test]
fn content_shorter_than_viewport_stays_put() {
    let mut s = ScrollView::new();
    s.set_viewport(26);
    s.set_total(10);
    let cursor = s.page_down(2);
    assert_eq!(s.offset(), 0);
    assert_eq!(cursor, 9);
}

#[test]
fn empty_content_keeps_cursor_at_zero() {
    let mut s = ScrollView::new();
    s.set_viewport(26);
    assert_eq!(s.page_down(0), 0);
    assert_eq!(s.page_up(0), 0);
}
