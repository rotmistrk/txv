//! Tests for InputLine navigation and selection.

use txv_core::event::{KeyCode, KeyEvent, KeyMod};
use txv_core::prelude::*;

use crate::InputLine;

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent {
        code,
        modifiers: KeyMod {
            ctrl: false,
            alt: false,
            shift: false,
        },
    })
}

fn shift_key(code: KeyCode) -> Event {
    Event::Key(KeyEvent {
        code,
        modifiers: KeyMod {
            ctrl: false,
            alt: false,
            shift: true,
        },
    })
}

// === Selection: Right/Left at boundaries ===

#[test]
fn right_at_end_clears_selection() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("hello");
    input.select_all();
    assert!(input.selection_range().is_some());
    input.handle(&key(KeyCode::Right));
    assert!(input.selection_range().is_none());
}

#[test]
fn left_clears_selection_and_moves() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("hello");
    input.select_all();
    input.handle(&key(KeyCode::Left));
    assert!(input.selection_range().is_none());
    assert_eq!(input.cursor, 4);
}

#[test]
fn left_at_zero_clears_selection() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("hi");
    input.selection = Some(2);
    input.cursor = 0;
    assert!(input.selection_range().is_some());
    input.handle(&key(KeyCode::Left));
    assert!(input.selection_range().is_none());
}

#[test]
fn right_without_selection_at_end_is_noop() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("ab");
    input.handle_nav(false, 2);
    let result = input.handle(&key(KeyCode::Right));
    assert_eq!(result, HandleResult::Consumed);
    assert_eq!(input.cursor, 2);
}

#[test]
fn left_without_selection_at_zero_is_noop() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("ab");
    input.handle_nav(false, 0);
    let result = input.handle(&key(KeyCode::Left));
    assert_eq!(result, HandleResult::Consumed);
    assert_eq!(input.cursor, 0);
}

// === Shift+Arrow extends selection ===

#[test]
fn shift_right_starts_selection() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("abc");
    input.handle_nav(false, 0);
    input.handle(&shift_key(KeyCode::Right));
    assert_eq!(input.selection_range(), Some((0, 1)));
    assert_eq!(input.cursor, 1);
}

#[test]
fn shift_left_starts_selection() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("abc");
    input.handle_nav(false, 2);
    input.handle(&shift_key(KeyCode::Left));
    assert_eq!(input.selection_range(), Some((1, 2)));
    assert_eq!(input.cursor, 1);
}

#[test]
fn shift_right_extends_selection() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("abcd");
    input.handle_nav(false, 1);
    input.handle(&shift_key(KeyCode::Right));
    input.handle(&shift_key(KeyCode::Right));
    assert_eq!(input.selection_range(), Some((1, 3)));
}

// === Home/End ===

#[test]
fn home_moves_to_start() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("hello");
    input.handle_nav(false, 3);
    input.handle(&key(KeyCode::Home));
    assert_eq!(input.cursor, 0);
    assert!(input.selection_range().is_none());
}

#[test]
fn end_moves_to_end() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("hello");
    input.handle_nav(false, 0);
    input.handle(&key(KeyCode::End));
    assert_eq!(input.cursor, 5);
    assert!(input.selection_range().is_none());
}

#[test]
fn shift_home_selects_to_start() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("hello");
    input.handle_nav(false, 3);
    input.handle(&shift_key(KeyCode::Home));
    assert_eq!(input.selection_range(), Some((0, 3)));
}

#[test]
fn shift_end_selects_to_end() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("hello");
    input.handle_nav(false, 2);
    input.handle(&shift_key(KeyCode::End));
    assert_eq!(input.selection_range(), Some((2, 5)));
}

// === select_all ===

#[test]
fn select_all_empty_does_nothing() {
    let mut input = InputLine::new().with_command(100);
    input.select_all();
    assert!(input.selection_range().is_none());
}

#[test]
fn select_all_selects_full_text() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("hello");
    input.select_all();
    assert_eq!(input.selection_range(), Some((0, 5)));
    assert_eq!(input.cursor, 5);
}

// === visible_start (scrolling) ===

#[test]
fn visible_start_zero_width_returns_zero() {
    let input = InputLine::new().with_command(100);
    assert_eq!(input.visible_start(0), 0);
}

#[test]
fn visible_start_short_text_returns_zero() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("hi");
    input.handle_nav(false, 2);
    assert_eq!(input.visible_start(10), 0);
}

#[test]
fn visible_start_scrolls_for_long_text() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("abcdefghij"); // 10 chars
    input.handle_nav(false, 9);
    let start = input.visible_start(5);
    assert!(start > 0);
    assert!(input.cursor >= start);
    assert!(input.cursor < start + 5);
}

// === cursor() ===

#[test]
fn cursor_returns_none_when_unfocused() {
    let input = InputLine::new().with_command(100);
    assert!(input.cursor().is_none());
}

#[test]
fn cursor_returns_position_when_focused() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("hello");
    input.state.set_focused(true);
    input.state.set_bounds(Rect::new(0, 0, 20, 1));
    input.handle_nav(false, 3);
    let req = input.cursor().unwrap();
    assert_eq!(req.x, 3);
    assert_eq!(req.y, 0);
}
