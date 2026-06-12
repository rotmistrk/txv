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
