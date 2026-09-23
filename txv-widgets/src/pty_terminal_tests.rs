//! Tests for PtyTerminal pinned scrollback behavior.

use txv_core::prelude::*;
use txv_render::termbuf::TermBuf;

use crate::pty_terminal::PtyTerminal;

/// Create a test terminal with no PTY (for scroll logic testing).
fn make_test_terminal(cols: u16, rows: u16, cursor_area: u16) -> PtyTerminal {
    PtyTerminal {
        state: ViewState::default(),
        termbuf: TermBuf::with_scrollback(cols, rows, 500),
        session: None,
        base_title: "test".to_string(),
        title: "test".to_string(),
        osc_suffix: String::new(),
        prev_cols: cols,
        prev_rows: rows,
        exited: true, // No session = exited
        scroll_offset: 0,
        had_output: false,
        pinned_mode: false,
        pinned_bottom_line: 0,
        cursor_area_lines: cursor_area,
    }
}

/// Feed lines to terminal to build up scrollback.
fn feed_lines(term: &mut PtyTerminal, count: usize) {
    for i in 0..count {
        let line = format!("Line {:03}\r\n", i);
        term.termbuf.process(line.as_bytes());
    }
}

/// Feed a wrapped line (longer than terminal width).
fn feed_wrapped_line(term: &mut PtyTerminal, text: &str) {
    let line = format!("{}\r\n", text);
    term.termbuf.process(line.as_bytes());
}

#[test]
fn initial_state_not_pinned() {
    let term = make_test_terminal(80, 24, 3);
    assert!(!term.pinned_mode);
    assert_eq!(term.scroll_offset, 0);
    assert_eq!(term.pinned_bottom_line, 0);
}

#[test]
fn page_up_enters_pinned_mode() {
    let mut term = make_test_terminal(80, 10, 3);
    feed_lines(&mut term, 50); // Lots of scrollback

    term.scroll_up_page();

    assert!(term.pinned_mode, "should enter pinned mode");
    assert!(term.pinned_bottom_line > 0, "should have pinned position");
}

#[test]
fn pinned_bottom_line_set_correctly_on_first_pgup() {
    let mut term = make_test_terminal(80, 10, 3);
    feed_lines(&mut term, 50);

    let total_before = term.termbuf.scrollback_len() + term.termbuf.grid_rows() as usize;
    term.scroll_up_page();

    // First PgUp should pin at total - cursor_area
    let expected_initial_pin = total_before - 3;
    // Then scroll up by page (scrollback_height - 1)
    let scrollback_height = 10 - 3 - 1; // h - cursor_area - separator = 6
    let page = scrollback_height - 1; // 5
    let expected = expected_initial_pin - page;

    assert_eq!(term.pinned_bottom_line, expected);
}

#[test]
fn new_output_does_not_change_pinned_bottom_line() {
    let mut term = make_test_terminal(80, 10, 3);
    feed_lines(&mut term, 50);

    term.scroll_up_page();
    let pinned_before = term.pinned_bottom_line;

    // Simulate new output
    feed_lines(&mut term, 5);

    // pinned_bottom_line should stay the same
    assert_eq!(
        term.pinned_bottom_line, pinned_before,
        "pinned position must not change"
    );
}

#[test]
fn gap_increases_with_new_output() {
    let mut term = make_test_terminal(80, 10, 3);
    feed_lines(&mut term, 50);

    term.scroll_up_page();
    let gap_before = term.calculate_displayed_gap();

    // Add more lines
    feed_lines(&mut term, 5);
    let gap_after = term.calculate_displayed_gap();

    assert!(
        gap_after > gap_before,
        "gap should increase: {} -> {}",
        gap_before,
        gap_after
    );
}

#[test]
fn page_down_moves_toward_live() {
    let mut term = make_test_terminal(80, 10, 3);
    feed_lines(&mut term, 100);

    // Scroll up multiple times
    term.scroll_up_page();
    term.scroll_up_page();
    term.scroll_up_page();
    let pinned_after_up = term.pinned_bottom_line;

    // Scroll down once
    term.scroll_down_page();
    let pinned_after_down = term.pinned_bottom_line;

    assert!(pinned_after_down > pinned_after_up, "should move toward newer content");
}

#[test]
fn page_down_exits_pinned_when_caught_up() {
    let mut term = make_test_terminal(80, 10, 3);
    feed_lines(&mut term, 30);

    term.scroll_up_page();
    assert!(term.pinned_mode);

    // Keep scrolling down until we exit pinned mode
    for _ in 0..20 {
        if !term.pinned_mode {
            break;
        }
        term.scroll_down_page();
    }

    assert!(!term.pinned_mode, "should exit pinned mode when caught up to live");
    assert_eq!(term.scroll_offset, 0);
}

#[test]
fn any_key_exits_pinned_mode() {
    let mut term = make_test_terminal(80, 10, 3);
    feed_lines(&mut term, 50);

    term.scroll_up_page();
    assert!(term.pinned_mode);

    // Send a regular key
    let key = KeyEvent::new(KeyCode::Char('a'), KeyMod::NONE);
    term.handle_key(&key);

    assert!(!term.pinned_mode, "regular key should exit pinned mode");
    assert_eq!(term.scroll_offset, 0);
}

#[test]
fn gap_accounts_for_wrapped_lines() {
    let mut term = make_test_terminal(20, 10, 3); // Narrow terminal
    feed_lines(&mut term, 20);

    // Feed a line that will wrap (longer than 20 cols)
    // "WRAPPED_LINE_THAT_IS_VERY_LONG" = 30 chars, wraps to 2 physical rows
    feed_wrapped_line(&mut term, "WRAPPED_LINE_THAT_IS_VERY_LONG_AND_CONTINUES");

    // Feed a few more normal lines
    feed_lines(&mut term, 5);

    term.scroll_up_page();
    term.scroll_up_page(); // Get the wrapped line in the gap

    let gap = term.calculate_displayed_gap();
    // Gap should count logical lines, so the wrapped line counts as 1
    // not as multiple physical rows
    assert!(gap > 0, "should have non-zero gap");
}

#[test]
fn scrollback_area_height_in_pinned_mode() {
    let mut term = make_test_terminal(80, 10, 3);
    feed_lines(&mut term, 50);

    // In pinned mode, scrollback area = h - cursor_area - separator = 10 - 3 - 1 = 6
    term.scroll_up_page();
    assert!(term.pinned_mode);

    // We can verify indirectly: page size should be scrollback_height - 1 = 5
    let pin_before = term.pinned_bottom_line;
    term.scroll_up_page();
    let pin_after = term.pinned_bottom_line;

    // Page moved us by at most 5 lines (may be clamped)
    let diff = pin_before.saturating_sub(pin_after);
    assert!(diff <= 5, "page should be at most 5, got {}", diff);
}

#[test]
fn minimum_pinned_bottom_line() {
    let mut term = make_test_terminal(80, 10, 3);
    feed_lines(&mut term, 100);

    // Scroll up many times
    for _ in 0..50 {
        term.scroll_up_page();
    }

    // Should not go below scrollback_height (need content to fill the view)
    let scrollback_height = 6; // 10 - 3 - 1
    assert!(
        term.pinned_bottom_line >= scrollback_height,
        "pinned_bottom_line {} should be >= {}",
        term.pinned_bottom_line,
        scrollback_height
    );
}

#[test]
fn page_size_based_on_scrollback_area() {
    let mut term = make_test_terminal(80, 10, 3);
    feed_lines(&mut term, 100);

    term.scroll_up_page();
    let first_pin = term.pinned_bottom_line;

    term.scroll_up_page();
    let second_pin = term.pinned_bottom_line;

    // Page size should be scrollback_height - 1 = 6 - 1 = 5
    let expected_page = 5;
    let actual_diff = first_pin - second_pin;

    // Allow for clamping at minimum
    assert!(
        actual_diff <= expected_page,
        "page diff {} should be <= expected {}",
        actual_diff,
        expected_page
    );
}

#[test]
fn extreme_wrap_logical_page_at_least_one() {
    // Narrow terminal where a single logical line can fill the gap
    let mut term = make_test_terminal(10, 10, 3); // 10 cols wide
    feed_lines(&mut term, 10);

    // Feed a line that wraps many times: 60 chars / 10 cols = 6 physical rows
    // This single logical line fills the entire scrollback area (6 rows)
    let long_line = "A".repeat(60);
    feed_wrapped_line(&mut term, &long_line);

    // Feed a few more normal lines to push the wrapped line into scrollback
    feed_lines(&mut term, 10);

    term.scroll_up_page();
    term.scroll_up_page(); // Try to get wrapped line in the gap

    let gap = term.calculate_displayed_gap();
    // Even if wrap fills the entire gap region, we should count logical lines
    // The gap should be >= 1 if there's any content in it
    assert!(
        gap >= 1 || term.pinned_bottom_line == 0,
        "gap should be at least 1 if there's content"
    );
}

#[test]
fn physical_scroll_always_progresses() {
    // Even when logical content is heavily wrapped, physical scroll always moves
    let mut term = make_test_terminal(10, 10, 3);
    feed_lines(&mut term, 50);

    term.scroll_up_page();
    let pin1 = term.pinned_bottom_line;

    term.scroll_up_page();
    let pin2 = term.pinned_bottom_line;

    // Physical page is at least 1 (max(scrollback_height - 1, 1))
    assert!(pin2 < pin1, "scroll should always move: {} -> {}", pin1, pin2);
}
