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

// ===== Command mode (: commands via keystrokes) =====

#[test]
fn command_mode_sort_via_keystrokes() {
    let mut te = TestEditor::new("cherry\napple\nbanana", 40, 10);
    te.feed(":%!sort\n");
    assert_eq!(te.content(), "apple\nbanana\ncherry\n");
}

#[test]
fn command_mode_substitute_via_keystrokes() {
    let mut te = TestEditor::new("hello world hello", 40, 10);
    te.feed(":s/hello/bye/g\n");
    assert_eq!(te.content(), "bye world bye");
}

#[test]
fn command_mode_set_nowrap_via_keystrokes() {
    let mut te = TestEditor::new("test", 40, 10);
    te.feed(":set nowrap\n");
    assert!(!te.ev.editor().options().wrap());
}

#[test]
fn command_mode_esc_cancels() {
    let mut te = TestEditor::new("original", 40, 10);
    te.feed(":dd");
    assert_eq!(te.ev.editor().mode(), EditorMode::Command);
    te.feed("\x1b");
    assert_eq!(te.ev.editor().mode(), EditorMode::Normal);
    assert_eq!(te.content(), "original"); // not executed
}

#[test]
fn command_mode_cursor_visible() {
    let mut te = TestEditor::new("test", 40, 10);
    te.feed(":");
    let c = te.ev.cursor();
    assert!(c.is_some(), "cursor should be visible in command mode");
}

// ===== Search via keystrokes =====

#[test]
fn search_forward_via_slash() {
    let mut te = TestEditor::new("alpha beta gamma beta", 60, 10);
    te.feed("/beta\n");
    assert_eq!(te.ev.editor().cursor_col(), 6);
}

#[test]
fn search_backward_via_question() {
    let mut te = TestEditor::new("alpha beta gamma beta end", 60, 10);
    te.feed("$"); // go to end
    let col_after_dollar = te.ev.editor().cursor_col();
    te.feed("?beta\n"); // search backward
    let col_after_search = te.ev.editor().cursor_col();
    assert_eq!(
        col_after_search, 17,
        "after $={col_after_dollar}, ?beta should find col 17, got {col_after_search}"
    );
}

#[test]
fn highlight_clears_on_motion() {
    let mut te = TestEditor::new("foo bar foo baz", 60, 10);
    te.feed("*"); // search "foo" — sets highlight
    assert!(te.ev.editor().highlight().is_some(), "highlight set after search");
    te.feed("j"); // motion — should clear
    assert!(te.ev.editor().highlight().is_none(), "highlight cleared on motion");
}

#[test]
fn highlight_persists_on_search_next() {
    let mut te = TestEditor::new("foo bar foo baz", 60, 10);
    te.feed("*"); // search "foo"
    te.feed("n"); // next — highlight should persist
    assert!(te.ev.editor().highlight().is_some(), "highlight kept on n");
}

#[test]
fn incremental_search_moves_cursor() {
    let mut te = TestEditor::new("aaa bbb ccc bbb", 60, 10);
    te.feed("/bb"); // don't press Enter yet
                    // Incremental search should have moved cursor to first "bb"
    assert_eq!(te.ev.editor().cursor_col(), 4);
    te.feed("\x1b"); // cancel — cursor returns to origin
    assert_eq!(te.ev.editor().cursor_col(), 0);
}

#[test]
fn search_shows_match_count() {
    let mut te = TestEditor::new("foo bar foo baz foo", 60, 10);
    te.feed("/foo"); // 3 matches
                     // Render and check last row for "3 found"
    te.ev.render();
    let buf = te.ev.buffer();
    let h = te.ev.bounds().h();
    let mut last_row = String::new();
    for x in 0..buf.width() {
        last_row.push(buf.cell(x, h - 1).ch());
    }
    assert!(last_row.starts_with('/'), "prefix should be /: {last_row}");
    assert!(last_row.contains("3 found"), "should show match count: {last_row}");
    te.feed("\x1b");
}

// ===== Software cursor visibility =====

#[test]
fn normal_mode_has_software_cursor() {
    let mut te = TestEditor::new("hello", 40, 10);
    // Hardware cursor should be None (software mode)
    assert!(te.ev.cursor().is_none(), "normal mode uses software cursor");
    // But the cell at cursor position should have flipped fg/bg
    te.ev.render();
    let buf = te.ev.buffer();
    let gw = te.ev.editor().options().number() as u16 * 2; // rough gutter
    let cell = buf.cell(gw, 0); // cursor at (gutter, 0)
    let normal_cell = buf.cell(gw + 1, 0); // next char
                                           // Cursor cell should have inverted colors vs normal cell
    assert_ne!(
        cell.style().bg(),
        normal_cell.style().bg(),
        "cursor cell bg should differ from neighbor"
    );
}

#[test]
fn insert_mode_has_hardware_cursor() {
    let mut te = TestEditor::new("hello", 40, 10);
    te.feed("i");
    let c = te.ev.cursor();
    assert!(c.is_some(), "insert mode should show hardware cursor");
}

// ===== Set options via command mode =====

#[test]
fn set_autoindent_off_disables_indent() {
    let mut te = TestEditor::new("    indented", 40, 10);
    te.feed(":set noai\n");
    te.feed("A\ntext\x1b");
    let c = te.content();
    // With noai, new line should not be auto-indented
    assert!(c.contains("\ntext"), "noai: new line should not be indented: {c}");
}

#[test]
fn set_paste_disables_autoindent() {
    let mut te = TestEditor::new("    indented", 40, 10);
    te.feed(":set paste\n");
    te.feed("A\ntext\x1b");
    let c = te.content();
    assert!(c.contains("\ntext"), "paste mode disables autoindent: {c}");
}

#[test]
fn set_expandtab() {
    let mut te = TestEditor::new("hello", 40, 10);
    te.feed(":set et\n");
    assert!(te.ev.editor().options().expandtab());
    te.feed(":set noet\n");
    assert!(!te.ev.editor().options().expandtab());
}

#[test]
fn set_shiftwidth() {
    let mut te = TestEditor::new("hello", 40, 10);
    te.feed(":set sw=2\n");
    assert_eq!(te.ev.editor().options().shiftwidth(), 2);
}

#[test]
fn set_tabstop() {
    let mut te = TestEditor::new("hello", 40, 10);
    te.feed(":set ts=8\n");
    assert_eq!(te.ev.editor().options().tab_width(), 8);
}

#[test]
fn set_hlsearch_off_clears_highlight() {
    let mut te = TestEditor::new("foo bar foo", 40, 10);
    te.feed("*"); // search — sets highlight
    assert!(te.ev.editor().highlight().is_some());
    te.feed(":set nohls\n");
    // Next motion should clear even though it's... wait, nohls means
    // search commands themselves don't keep highlight
    te.feed("n");
    assert!(
        te.ev.editor().highlight().is_none(),
        "nohls prevents highlight retention"
    );
}
