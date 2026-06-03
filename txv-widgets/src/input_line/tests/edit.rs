//! Tests for InputLine editing, history, and key dispatch.

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

fn ctrl_key(ch: char) -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Char(ch),
        modifiers: KeyMod {
            ctrl: true,
            alt: false,
            shift: false,
        },
    })
}

fn alt_key(ch: char) -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Char(ch),
        modifiers: KeyMod {
            ctrl: false,
            alt: true,
            shift: false,
        },
    })
}

fn char_key(ch: char) -> Event {
    Event::Key(KeyEvent {
        code: KeyCode::Char(ch),
        modifiers: KeyMod {
            ctrl: false,
            alt: false,
            shift: false,
        },
    })
}

// === Character insertion ===

#[test]
fn char_inserts_at_cursor() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("ac");
    input.handle_nav(false, 1);
    input.handle(&char_key('b'));
    assert_eq!(input.text(), "abc");
    assert_eq!(input.cursor, 2);
}

#[test]
fn char_replaces_selection() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("hello");
    input.select_all();
    input.handle(&char_key('x'));
    assert_eq!(input.text(), "x");
    assert_eq!(input.cursor, 1);
    assert!(input.selection_range().is_none());
}

// === Backspace ===

#[test]
fn backspace_deletes_char_before_cursor() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("abc");
    input.handle_nav(false, 2);
    input.handle(&key(KeyCode::Backspace));
    assert_eq!(input.text(), "ac");
    assert_eq!(input.cursor, 1);
}

#[test]
fn backspace_at_zero_is_noop() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("abc");
    input.handle_nav(false, 0);
    input.handle(&key(KeyCode::Backspace));
    assert_eq!(input.text(), "abc");
    assert_eq!(input.cursor, 0);
}

#[test]
fn backspace_deletes_selection() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("hello");
    input.select_all();
    input.handle(&key(KeyCode::Backspace));
    assert_eq!(input.text(), "");
    assert_eq!(input.cursor, 0);
}

// === Delete ===

#[test]
fn delete_removes_char_at_cursor() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("abc");
    input.handle_nav(false, 1);
    input.handle(&key(KeyCode::Delete));
    assert_eq!(input.text(), "ac");
    assert_eq!(input.cursor, 1);
}

#[test]
fn delete_at_end_is_noop() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("abc");
    input.handle_nav(false, 3);
    input.handle(&key(KeyCode::Delete));
    assert_eq!(input.text(), "abc");
}

#[test]
fn delete_removes_selection() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("hello");
    input.handle_nav(false, 1);
    input.handle_nav(true, 4);
    input.handle(&key(KeyCode::Delete));
    assert_eq!(input.text(), "ho");
    assert_eq!(input.cursor, 1);
}

// === History ===

#[test]
fn history_up_recalls_last_entry() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("first");
    input.push_history();
    input.set_text("second");
    input.push_history();
    input.set_text("");
    input.handle(&key(KeyCode::Up));
    assert_eq!(input.text(), "second");
    input.handle(&key(KeyCode::Up));
    assert_eq!(input.text(), "first");
}

#[test]
fn history_up_at_top_stays() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("only");
    input.push_history();
    input.set_text("");
    input.handle(&key(KeyCode::Up));
    assert_eq!(input.text(), "only");
    input.handle(&key(KeyCode::Up));
    assert_eq!(input.text(), "only");
}

#[test]
fn history_down_after_up_restores() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("first");
    input.push_history();
    input.set_text("second");
    input.push_history();
    input.set_text("");
    input.handle(&key(KeyCode::Up));
    input.handle(&key(KeyCode::Up));
    input.handle(&key(KeyCode::Down));
    assert_eq!(input.text(), "second");
    input.handle(&key(KeyCode::Down));
    assert_eq!(input.text(), "");
}

#[test]
fn history_down_without_up_is_noop() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("hello");
    input.handle(&key(KeyCode::Down));
    assert_eq!(input.text(), "hello");
}

#[test]
fn history_up_empty_is_noop() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("hello");
    input.handle(&key(KeyCode::Up));
    assert_eq!(input.text(), "hello");
}

// === Ctrl keys ===

#[test]
fn ctrl_c_with_selection_emits_copy() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("hello");
    input.select_all();
    let result = input.handle(&ctrl_key('c'));
    assert_eq!(result, HandleResult::Consumed);
}

#[test]
fn ctrl_v_emits_paste_request() {
    let mut input = InputLine::new().with_command(100);
    let result = input.handle(&ctrl_key('v'));
    assert_eq!(result, HandleResult::Consumed);
}

#[test]
fn ctrl_other_is_ignored() {
    let mut input = InputLine::new().with_command(100);
    let result = input.handle(&ctrl_key('z'));
    assert_eq!(result, HandleResult::Ignored);
}

#[test]
fn alt_key_is_ignored() {
    let mut input = InputLine::new().with_command(100);
    let result = input.handle(&alt_key('x'));
    assert_eq!(result, HandleResult::Ignored);
}

// === Esc / Unknown / Non-key ===

#[test]
fn esc_emits_cancel() {
    let mut input = InputLine::new().with_command(100);
    let result = input.handle(&key(KeyCode::Esc));
    assert_eq!(result, HandleResult::Consumed);
}

#[test]
fn unknown_key_is_ignored() {
    let mut input = InputLine::new().with_command(100);
    let result = input.handle(&key(KeyCode::F(12)));
    assert_eq!(result, HandleResult::Ignored);
}

#[test]
fn tick_event_is_ignored() {
    let mut input = InputLine::new().with_command(100);
    let result = input.handle(&Event::Tick);
    assert_eq!(result, HandleResult::Ignored);
}

// === insert_text ===

#[test]
fn insert_text_replaces_selection() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("hello world");
    input.handle_nav(false, 0);
    input.handle_nav(true, 5);
    input.insert_text("bye");
    assert_eq!(input.text(), "bye world");
    assert_eq!(input.cursor, 3);
}

// === Unicode ===

#[test]
fn unicode_char_handling() {
    let mut input = InputLine::new().with_command(100);
    input.set_text("héllo");
    assert_eq!(input.char_count(), 5);
    input.handle_nav(false, 2);
    input.handle(&key(KeyCode::Backspace));
    assert_eq!(input.text(), "hllo");
}
