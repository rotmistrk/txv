//! Editor vi-mode integration tests — full keystroke simulation via EditorView.
//!
//! Complete path: key → keymap → command → execute → buffer.
//! Command mode uses InputLine child — sink events (CM_OK, CM_CANCEL) are
//! drained and re-dispatched after each key.

use txv_core::prelude::*;
use txv_edit::editor::keymap::EditorMode;
use txv_edit::view::EditorView;

struct TestEditor {
    ev: EditorView,
    sink: EventSink,
}

impl TestEditor {
    fn new(content: &str, w: u16, h: u16) -> Self {
        let mut ev = EditorView::from_text(content);
        ev.set_bounds(Rect::new(0, 0, w, h));
        let sink = EventSink::new();
        ev.set_sink(sink.clone());
        Self { ev, sink }
    }

    fn feed(&mut self, keys: &str) {
        for ch in keys.chars() {
            let code = match ch {
                '\x1b' => KeyCode::Esc,
                '\n' => KeyCode::Enter,
                _ => KeyCode::Char(ch),
            };
            self.ev.handle(&Event::Key(KeyEvent::new(code, KeyMod::NONE)));
            self.drain();
        }
    }

    fn feed_key(&mut self, code: KeyCode, mods: KeyMod) {
        self.ev.handle(&Event::Key(KeyEvent::new(code, mods)));
        self.drain();
    }

    fn drain(&mut self) {
        loop {
            let events = self.sink.drain();
            if events.is_empty() {
                return;
            }
            for e in events {
                self.ev.handle(&e);
            }
        }
    }

    fn content(&self) -> String {
        self.ev.content()
    }
}

// ===== Motions =====

#[test]
fn goto_line_22g() {
    let text = (1..=30).map(|i| format!("line {i}")).collect::<Vec<_>>().join("\n");
    let mut te = TestEditor::new(&text, 40, 10);
    te.feed("22G");
    assert_eq!(te.ev.editor().cursor_line(), 21);
}

#[test]
fn move_to_end_of_file() {
    let mut te = TestEditor::new("a\nb\nc\nd\ne", 40, 10);
    te.feed("G");
    assert_eq!(te.ev.editor().cursor_line(), 4);
}

#[test]
fn move_to_start_of_file() {
    let mut te = TestEditor::new("a\nb\nc", 40, 10);
    te.feed("Ggg");
    assert_eq!(te.ev.editor().cursor_line(), 0);
}

// ===== Yank and Paste =====

#[test]
fn yank_line_and_paste() {
    let mut te = TestEditor::new("alpha\nbeta\ngamma", 40, 10);
    te.feed("yy");
    te.feed("j");
    te.feed("p");
    assert_eq!(te.content(), "alpha\nbeta\nalpha\ngamma");
}

#[test]
fn paste_before_with_capital_p() {
    let mut te = TestEditor::new("alpha\nbeta\ngamma", 40, 10);
    te.feed("jyy");
    te.feed("gg");
    te.feed("P");
    assert_eq!(te.content(), "beta\nalpha\nbeta\ngamma");
}

// ===== Visual modes =====

#[test]
fn visual_line_select_and_yank() {
    let mut te = TestEditor::new("one\ntwo\nthree\nfour", 40, 10);
    te.feed("V");
    assert_eq!(te.ev.editor().mode(), EditorMode::VisualLine);
    te.feed("j");
    te.feed("y");
    assert_eq!(te.ev.editor().mode(), EditorMode::Normal);
    assert!(te.ev.editor().register().contains("one\ntwo\n"));
}

#[test]
fn visual_char_select_and_yank() {
    let mut te = TestEditor::new("hello world", 40, 10);
    te.feed("v");
    assert_eq!(te.ev.editor().mode(), EditorMode::Visual);
    te.feed("llll");
    te.feed("y");
    assert_eq!(te.ev.editor().register(), "hello");
}

#[test]
fn visual_block_select() {
    let mut te = TestEditor::new("abcd\nefgh\nijkl", 40, 10);
    te.feed_key(KeyCode::Char('v'), KeyMod::CTRL);
    assert_eq!(te.ev.editor().mode(), EditorMode::VisualBlock);
    te.feed("jl");
    te.feed("y");
    assert!(te.ev.editor().register_block());
    assert_eq!(te.ev.editor().register(), "ab\nef");
}

// ===== Delete and Change =====

#[test]
fn delete_line_dd() {
    let mut te = TestEditor::new("one\ntwo\nthree", 40, 10);
    te.feed("jdd");
    assert_eq!(te.content(), "one\nthree");
}

#[test]
fn change_word_cw() {
    let mut te = TestEditor::new("hello world", 40, 10);
    te.feed("cw");
    te.feed("goodbye\x1b");
    assert_eq!(te.content(), "goodbyeworld");
}

// ===== Undo/Redo =====

#[test]
fn undo_restores_content() {
    let mut te = TestEditor::new("original", 40, 10);
    te.feed("dd");
    assert_eq!(te.content(), "");
    te.feed("u");
    assert_eq!(te.content(), "original");
}

#[test]
fn redo_after_undo() {
    let mut te = TestEditor::new("original", 40, 10);
    te.feed("dd");
    te.feed("u");
    te.feed_key(KeyCode::Char('r'), KeyMod::CTRL);
    assert_eq!(te.content(), "");
}

// ===== Insert mode =====

#[test]
fn insert_mode_typing() {
    let mut te = TestEditor::new("", 40, 10);
    te.feed("i");
    assert_eq!(te.ev.editor().mode(), EditorMode::Insert);
    te.feed("hello\x1b");
    assert_eq!(te.content(), "hello");
    assert_eq!(te.ev.editor().mode(), EditorMode::Normal);
}

// ===== Search =====

#[test]
fn search_forward_via_keys() {
    let mut te = TestEditor::new("foo bar foo baz foo", 60, 10);
    // '/' enters search mode, but EditorView can't handle typing in search mode
    // (same limitation as command mode). Test the motion directly.
    te.feed("*"); // search word under cursor (foo)
    let col = te.ev.editor().cursor_col();
    assert_eq!(col, 8, "* should find next 'foo' at col 8");
}

#[test]
fn search_next_with_n() {
    let mut te = TestEditor::new("foo bar foo baz foo", 60, 10);
    te.feed("*"); // search "foo", lands at col 8
    te.feed("n"); // next match at col 16
    assert_eq!(te.ev.editor().cursor_col(), 16);
}

#[test]
fn search_prev_with_capital_n() {
    let mut te = TestEditor::new("foo bar foo baz foo", 60, 10);
    te.feed("*"); // col 8
    te.feed("n"); // col 16
    te.feed("N"); // back to col 8
    assert_eq!(te.ev.editor().cursor_col(), 8);
}

// ===== Marks =====

#[test]
fn set_and_jump_to_mark() {
    let mut te = TestEditor::new("line1\nline2\nline3", 40, 10);
    te.feed("j");
    te.feed("ma");
    te.feed("gg");
    te.feed("'a");
    assert_eq!(te.ev.editor().cursor_line(), 1);
}

// ===== Match bracket =====

#[test]
fn match_bracket_percent() {
    let mut te = TestEditor::new("(hello)", 40, 10);
    te.feed("%");
    assert_eq!(te.ev.editor().cursor_col(), 6);
    te.feed("%");
    assert_eq!(te.ev.editor().cursor_col(), 0);
}
