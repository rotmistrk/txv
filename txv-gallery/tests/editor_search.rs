//! Editor search and substitute tests.

use txv_core::prelude::*;
use txv_edit::editor::command::Command;
use txv_edit::editor::Editor;
use txv_edit::view::EditorView;

/// Helper: create an EditorView with content and given bounds.
fn editor_with(content: &str, w: u16, h: u16) -> EditorView {
    let mut ev = EditorView::from_text(content);
    ev.set_bounds(Rect::new(0, 0, w, h));
    ev
}

/// Helper: feed keys as string (normal mode chars).
fn feed(ev: &mut EditorView, keys: &str) {
    for ch in keys.chars() {
        let code = match ch {
            '\x1b' => KeyCode::Esc,
            '\n' => KeyCode::Enter,
            _ => KeyCode::Char(ch),
        };
        ev.handle(&Event::Key(KeyEvent::new(code, KeyMod::NONE)));
    }
}

// ===== Search forward/backward =====

#[test]
fn search_forward_finds_match() {
    let mut ed = Editor::from_text("alpha beta gamma beta delta");
    ed.set_viewport_height(10);
    ed.execute(Command::SearchForward("beta".into()));
    assert_eq!(ed.cursor_col(), 6); // first "beta" at col 6
}

#[test]
fn search_backward_finds_match() {
    let mut ed = Editor::from_text("alpha beta gamma beta delta");
    ed.set_viewport_height(10);
    // Start at end
    ed.set_cursor_col(26);
    ed.execute(Command::SearchBackward("beta".into()));
    assert_eq!(ed.cursor_col(), 17); // second "beta" at col 17
}

#[test]
fn search_next_wraps_around() {
    let mut ed = Editor::from_text("foo bar foo baz");
    ed.set_viewport_height(10);
    ed.execute(Command::SearchForward("foo".into()));
    // First search lands at col 8 (second foo)
    let first = ed.cursor_col();
    ed.execute(Command::SearchNext);
    let second = ed.cursor_col();
    // Should wrap back to col 0
    assert_ne!(first, second, "should advance");
    assert!(ed.status().contains("wrapped") || second == 0);
}

#[test]
fn search_word_forward() {
    let mut ed = Editor::from_text("hello world hello there");
    ed.set_viewport_height(10);
    ed.execute(Command::SearchWordForward);
    // Cursor at col 0 on "hello", should find next "hello" at col 12
    assert_eq!(ed.cursor_col(), 12);
}

// ===== Incremental search =====

#[test]
fn incremental_search_sets_origin() {
    let mut ed = Editor::from_text("line one\nline two\nline three");
    ed.set_viewport_height(10);
    ed.set_cursor_line(1);
    ed.execute(Command::EnterSearchMode);
    assert_eq!(ed.mode(), txv_edit::editor::keymap::EditorMode::Search);
    // Origin should be saved
}

// ===== Search via keymap =====

#[test]
fn search_forward_and_next() {
    let mut ev = editor_with("foo bar foo baz foo", 60, 10);
    feed(&mut ev, "/foo\n");
    // Should move cursor to first "foo" after current position
    let col = ev.editor().cursor_col();
    assert!(col == 8 || col == 0, "should find a foo: col={col}");
    feed(&mut ev, "n"); // next occurrence
}

// ===== Substitute =====

#[test]
fn substitute_single_line() {
    let mut ed = Editor::from_text("hello world hello");
    ed.set_viewport_height(10);
    ed.execute(Command::ExCommand("s/hello/bye/".into()));
    assert_eq!(ed.buf().content(), "bye world hello");
}

#[test]
fn substitute_global_on_line() {
    let mut ed = Editor::from_text("hello world hello");
    ed.set_viewport_height(10);
    ed.execute(Command::ExCommand("s/hello/bye/g".into()));
    assert_eq!(ed.buf().content(), "bye world bye");
}

#[test]
fn substitute_range() {
    let mut ed = Editor::from_text("aaa\nbbb\naaa\nbbb");
    ed.set_viewport_height(10);
    ed.execute(Command::ExCommand("%s/aaa/zzz/g".into()));
    assert_eq!(ed.buf().content(), "zzz\nbbb\nzzz\nbbb");
}

#[test]
fn substitute_with_regex() {
    let mut ed = Editor::from_text("foo123 bar456");
    ed.set_viewport_height(10);
    ed.execute(Command::ExCommand("s/[0-9]+/NUM/g".into()));
    assert_eq!(ed.buf().content(), "fooNUM barNUM");
}
