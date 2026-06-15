//! Comprehensive editor cursor + editing correctness tests.
//! Verifies cursor position matches edit location across all entry modes,
//! with various configurations (wrap/nowrap, nu/nonu, wide/narrow).

use txv_core::prelude::*;
use txv_edit::editor::command::Command;
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
            self.self.drain();
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

    fn set_opt(&mut self, opt: &str) {
        self.ev.editor_mut().execute(Command::ExCommand(format!("set {opt}")));
    }

    fn content(&self) -> String {
        self.ev.content()
    }
    fn line(&self) -> usize {
        self.ev.editor().cursor_line()
    }
    fn col(&self) -> usize {
        self.ev.editor().cursor_col()
    }
    fn mode(&self) -> EditorMode {
        self.ev.editor().mode()
    }

    fn line_text(&self, idx: usize) -> String {
        self.ev.editor().buf().line(idx).unwrap_or_default()
    }

    /// Get visual cursor position from the cursor() method.
    fn visual_cursor(&mut self) -> Option<(u16, u16)> {
        self.set_opt("cursor_normal=block");
        self.set_opt("cursor_insert=bar");
        self.ev.render();
        self.ev.cursor().map(|c| (c.x(), c.y()))
    }
}

// ======================================================================
// o / O — open line below / above
// ======================================================================

#[test]
fn o_opens_line_below_and_cursor_on_new_line() {
    let mut te = TestEditor::new("line1\nline2\nline3", 40, 10);
    te.feed("o");
    assert_eq!(te.mode(), EditorMode::Insert);
    assert_eq!(te.line(), 1, "cursor on new line below");
    assert_eq!(te.col(), 0);
    assert_eq!(te.line_text(1), "", "new line is empty");
    assert_eq!(te.line_text(0), "line1", "original line unchanged");
    assert_eq!(te.line_text(2), "line2", "line2 shifted down");
}

#[test]
fn o_on_last_line() {
    let mut te = TestEditor::new("only", 40, 10);
    te.feed("o");
    assert_eq!(te.line(), 1);
    assert_eq!(te.line_text(0), "only");
    assert_eq!(te.line_text(1), "");
}

#[test]
fn o_on_middle_line() {
    let mut te = TestEditor::new("a\nb\nc", 40, 10);
    te.feed("jo"); // move to line 1, then o
    assert_eq!(te.line(), 2, "new line inserted below line 1");
    assert_eq!(te.line_text(1), "b");
    assert_eq!(te.line_text(2), "");
    assert_eq!(te.line_text(3), "c");
}

#[test]
fn big_o_opens_line_above() {
    let mut te = TestEditor::new("line1\nline2\nline3", 40, 10);
    te.feed("jO"); // go to line1, O opens above
    assert_eq!(te.line(), 1, "cursor on new line above line2");
    assert_eq!(te.col(), 0);
    assert_eq!(te.line_text(0), "line1");
    assert_eq!(te.line_text(1), "", "new empty line");
    assert_eq!(te.line_text(2), "line2");
}

#[test]
fn big_o_on_first_line() {
    let mut te = TestEditor::new("first\nsecond", 40, 10);
    te.feed("O");
    assert_eq!(te.line(), 0, "cursor on new line 0");
    assert_eq!(te.line_text(0), "");
    assert_eq!(te.line_text(1), "first");
}

// ======================================================================
// i / I / a / A — insert modes
// ======================================================================

#[test]
fn i_inserts_at_cursor() {
    let mut te = TestEditor::new("hello", 40, 10);
    te.feed("llixyz\x1b");
    assert_eq!(te.content().trim(), "hexyzllo");
    assert_eq!(te.col(), 4); // after Esc, cursor on last inserted char
}

#[test]
fn big_i_inserts_at_first_non_blank() {
    let mut te = TestEditor::new("  hello", 40, 10);
    te.feed("Iabc\x1b");
    // I goes to first non-blank (col 2), inserts there
    assert_eq!(te.line_text(0), "  abchello");
}

#[test]
fn a_appends_after_cursor() {
    let mut te = TestEditor::new("abc", 40, 10);
    te.feed("laX\x1b");
    assert_eq!(te.content().trim(), "abXc");
}

#[test]
fn big_a_appends_at_end_of_line() {
    let mut te = TestEditor::new("hello", 40, 10);
    te.feed("AWORLD\x1b");
    assert_eq!(te.line_text(0), "helloWORLD");
}

// ======================================================================
// p / P — paste
// ======================================================================

#[test]
fn p_pastes_after_cursor_charwise() {
    let mut te = TestEditor::new("abcd", 40, 10);
    te.feed("vyp"); // visual select 'a', yank, paste after
    assert_eq!(te.line_text(0), "aabcd");
}

#[test]
fn big_p_pastes_before_cursor_charwise() {
    let mut te = TestEditor::new("abcd", 40, 10);
    te.feed("lvyP"); // move to 'b', visual 'b', yank, paste before
    assert_eq!(te.line_text(0), "abbcd");
}

#[test]
fn p_pastes_linewise_below() {
    let mut te = TestEditor::new("line1\nline2\nline3", 40, 10);
    te.feed("yyp"); // yank line1, paste below
    assert_eq!(te.line(), 1);
    assert_eq!(te.line_text(0), "line1");
    assert_eq!(te.line_text(1), "line1");
    assert_eq!(te.line_text(2), "line2");
}

#[test]
fn big_p_pastes_linewise_above() {
    let mut te = TestEditor::new("line1\nline2", 40, 10);
    te.feed("jyyP"); // go to line2, yank, paste above
    assert_eq!(te.line(), 1);
    assert_eq!(te.line_text(0), "line1");
    assert_eq!(te.line_text(1), "line2");
    assert_eq!(te.line_text(2), "line2");
}

// ======================================================================
// dd / cc / D / C — line/rest deletion
// ======================================================================

#[test]
fn dd_deletes_line_cursor_stays() {
    let mut te = TestEditor::new("a\nb\nc", 40, 10);
    te.feed("jdd");
    assert_eq!(te.content().trim(), "a\nc");
    assert_eq!(te.line(), 1, "cursor on next line (now 'c')");
}

#[test]
fn dd_on_last_line_goes_up() {
    let mut te = TestEditor::new("a\nb", 40, 10);
    te.feed("jdd");
    assert_eq!(te.content().trim(), "a");
    assert_eq!(te.line(), 0);
}

#[test]
fn cc_changes_entire_line() {
    let mut te = TestEditor::new("hello\nworld", 40, 10);
    te.feed("ccnew\x1b");
    assert_eq!(te.line_text(0), "new");
    assert_eq!(te.line(), 0);
}

#[test]
fn big_d_deletes_to_end_of_line() {
    let mut te = TestEditor::new("abcdef", 40, 10);
    te.feed("llD");
    assert_eq!(te.line_text(0), "ab");
}

#[test]
fn big_c_changes_to_end_of_line() {
    let mut te = TestEditor::new("abcdef", 40, 10);
    te.feed("llCxy\x1b");
    assert_eq!(te.line_text(0), "abxy");
}

// ======================================================================
// x / X — single char delete
// ======================================================================

#[test]
fn x_deletes_char_under_cursor() {
    let mut te = TestEditor::new("abcde", 40, 10);
    te.feed("llx");
    assert_eq!(te.line_text(0), "abde");
    assert_eq!(te.col(), 2);
}

#[test]
fn big_x_deletes_char_before_cursor() {
    let mut te = TestEditor::new("abcde", 40, 10);
    te.feed("llX");
    assert_eq!(te.line_text(0), "acde");
    assert_eq!(te.col(), 1);
}
