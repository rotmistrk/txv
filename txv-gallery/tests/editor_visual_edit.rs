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
// Visual mode selections
// ======================================================================

#[test]
fn visual_line_delete() {
    let mut te = TestEditor::new("a\nb\nc\nd", 40, 10);
    te.feed("jVjd"); // select lines 1-2, delete
    assert_eq!(te.content().trim(), "a\nd");
    assert_eq!(te.line(), 1);
}

#[test]
fn visual_char_yank_and_paste() {
    let mut te = TestEditor::new("hello world", 40, 10);
    te.feed("wvlly"); // w→col6('w'), v select, ll→col8, yank "wor"
    te.feed("$p"); // go to end, paste
    assert_eq!(te.line_text(0), "hello worldwor");
}

#[test]
fn visual_block_insert() {
    let mut te = TestEditor::new("aaa\nbbb\nccc", 40, 10);
    te.feed_key(KeyCode::Char('v'), KeyMod::CTRL); // C-v block select
    te.feed("jjI#\x1b"); // select 3 rows, insert # at start
    assert_eq!(te.line_text(0), "#aaa");
    assert_eq!(te.line_text(1), "#bbb");
    assert_eq!(te.line_text(2), "#ccc");
}

#[test]
fn visual_block_delete() {
    let mut te = TestEditor::new("abcd\nefgh\nijkl", 40, 10);
    te.feed_key(KeyCode::Char('v'), KeyMod::CTRL); // C-v
    te.feed("ljjd"); // select 2 cols × 3 rows, delete
    assert_eq!(te.line_text(0), "cd");
    assert_eq!(te.line_text(1), "gh");
    assert_eq!(te.line_text(2), "kl");
}

// ======================================================================
// Cursor position consistency with visual cursor
// ======================================================================

#[test]
fn cursor_visual_matches_logical_after_o() {
    let mut te = TestEditor::new("line1\nline2\nline3", 60, 20);
    te.set_opt("nonu");
    te.feed("jo"); // go to line 1, open below
    let (_, vy) = te.visual_cursor().expect("cursor visible");
    // Cursor should be on visual row 2 (line 0=row0, line1=row1, new=row2)
    assert_eq!(vy, 2, "visual cursor matches logical line");
    assert_eq!(te.line(), 2);
}

#[test]
fn cursor_visual_matches_logical_after_big_o() {
    let mut te = TestEditor::new("line1\nline2\nline3", 60, 20);
    te.set_opt("nonu");
    te.feed("jO"); // go to line 1, open above
    let (_, vy) = te.visual_cursor().expect("cursor visible");
    assert_eq!(vy, 1, "visual cursor on inserted line above line2");
    assert_eq!(te.line(), 1);
}

// ======================================================================
// With wrap enabled
// ======================================================================

#[test]
fn o_with_wrap_cursor_correct() {
    // Long lines that wrap — cursor must still land on the right visual row
    let long = "x".repeat(50);
    let content = format!("{long}\n{long}\nshort");
    let mut te = TestEditor::new(&content, 30, 20);
    te.set_opt("wrap");
    te.set_opt("nonu");
    te.feed("o"); // open line below line 0
    assert_eq!(te.line(), 1);
    assert_eq!(te.line_text(1), "");
    assert_eq!(te.mode(), EditorMode::Insert);
}

#[test]
fn dd_with_wrap_cursor_correct() {
    let long = "x".repeat(50);
    let content = format!("{long}\nshort\n{long}");
    let mut te = TestEditor::new(&content, 30, 20);
    te.set_opt("wrap");
    te.feed("jdd"); // delete "short"
    assert_eq!(te.line(), 1);
    assert!(te.line_text(1).starts_with("xxx"));
}

// ======================================================================
// With line numbers (nu)
// ======================================================================

#[test]
fn o_with_nu_cursor_correct() {
    let mut te = TestEditor::new("one\ntwo\nthree", 60, 20);
    te.set_opt("nu");
    te.feed("jo");
    assert_eq!(te.line(), 2);
    assert_eq!(te.line_text(2), "");
    // Visual cursor X should account for gutter
    if let Some((vx, _)) = te.visual_cursor() {
        assert!(vx >= 2, "cursor past gutter, got x={vx}");
    }
}

// ======================================================================
// Narrow terminal
// ======================================================================

#[test]
fn editing_in_narrow_terminal() {
    let mut te = TestEditor::new("abcdef\nghijkl", 10, 5);
    te.set_opt("nowrap");
    te.set_opt("nonu");
    te.feed("llllllaINS\x1b"); // move to col 6, append "INS"
    assert_eq!(te.line_text(0), "abcdefINS");
    // Cursor should be within bounds
    assert!(te.col() <= 10);
}

#[test]
fn o_in_narrow_terminal() {
    let mut te = TestEditor::new("abc\ndef", 10, 5);
    te.set_opt("nonu");
    te.feed("ohello\x1b");
    assert_eq!(te.line(), 1);
    assert_eq!(te.line_text(1), "hello");
    assert_eq!(te.line_text(2), "def");
}

// ======================================================================
// Multi-sequence operations
// ======================================================================

#[test]
fn undo_redo_cursor_consistency() {
    let mut te = TestEditor::new("start", 40, 10);
    te.feed("Amore\x1b"); // append "more" → "startmore"
    assert_eq!(te.line_text(0), "startmore");
    te.feed("u"); // undo
    assert_eq!(te.line_text(0), "start");
    te.feed_key(KeyCode::Char('r'), KeyMod::CTRL); // redo
    assert_eq!(te.line_text(0), "startmore");
}

#[test]
fn multiple_o_sequence() {
    let mut te = TestEditor::new("top\nbottom", 40, 10);
    te.feed("oA\x1b"); // open below top, type A
    te.feed("oB\x1b"); // open below A, type B
    te.feed("oC\x1b"); // open below B, type C
    assert_eq!(te.line_text(0), "top");
    assert_eq!(te.line_text(1), "A");
    assert_eq!(te.line_text(2), "B");
    assert_eq!(te.line_text(3), "C");
    assert_eq!(te.line_text(4), "bottom");
    assert_eq!(te.line(), 3); // cursor on C
}

#[test]
fn dd_then_p_restores() {
    let mut te = TestEditor::new("a\nb\nc", 40, 10);
    te.feed("jdd"); // delete line "b"
    assert_eq!(te.content().trim(), "a\nc");
    te.feed("P"); // paste above
    assert_eq!(te.line_text(0), "a");
    assert_eq!(te.line_text(1), "b");
    assert_eq!(te.line_text(2), "c");
}

#[test]
fn visual_line_change() {
    let mut te = TestEditor::new("line1\nline2\nline3", 40, 10);
    te.feed("jVc"); // select line2, change
    te.feed("replaced\x1b");
    assert_eq!(te.line_text(0), "line1");
    assert_eq!(te.line_text(1), "replaced");
    assert_eq!(te.line_text(2), "line3");
}

#[test]
fn insert_at_end_then_navigate() {
    let mut te = TestEditor::new("abc\ndef\nghi", 40, 10);
    te.feed("GA\nNEW\x1b"); // go to end, append newline + NEW
    assert_eq!(te.line(), 3);
    assert_eq!(te.line_text(3), "NEW");
    te.feed("gg"); // go back to top
    assert_eq!(te.line(), 0);
    assert_eq!(te.col(), 0);
}
