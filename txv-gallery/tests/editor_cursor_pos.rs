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
// Visual cursor Y position correctness (the original bug: o showed cursor on wrong line)
// ======================================================================

#[test]
fn visual_cursor_y_after_o_no_wrap() {
    let mut te = TestEditor::new("line1\nline2\nline3", 60, 20);
    te.set_opt("nonu");
    te.feed("jo"); // go to line 1, open below → cursor on line 2
    let (_, vy) = te.visual_cursor().expect("cursor visible");
    assert_eq!(te.line(), 2);
    assert_eq!(vy, 2, "visual Y matches logical line (no wrap, no scroll)");
}

#[test]
fn visual_cursor_y_after_o_with_wrap() {
    // First line wraps to 2 visual rows in 20-col viewport
    let content = format!("{}\nshort\nend", "x".repeat(40));
    let mut te = TestEditor::new(&content, 20, 20);
    te.set_opt("wrap");
    te.set_opt("nonu");
    te.feed("jo"); // go to "short" (line 1), open below → cursor on line 2
    let (_, vy) = te.visual_cursor().expect("cursor visible");
    assert_eq!(te.line(), 2);
    // Line 0 wraps to 2 rows (40/20=2), line 1 = 1 row, new line 2 = row 3
    assert_eq!(vy, 3, "visual Y accounts for wrapped line above");
}

#[test]
fn visual_cursor_y_with_multiple_wrapped_lines() {
    // 3 lines each 60 chars → 3 visual rows each in 20-col viewport
    let lines: Vec<String> = (0..3).map(|i| format!("{}", (b'a' + i) as char).repeat(60)).collect();
    let content = lines.join("\n");
    let mut te = TestEditor::new(&content, 20, 30);
    te.set_opt("wrap");
    te.set_opt("nonu");
    te.feed("jj"); // go to line 2
    let (_, vy) = te.visual_cursor().expect("cursor visible");
    assert_eq!(te.line(), 2);
    // Line 0: 3 rows (60/20), line 1: 3 rows → cursor at visual row 6
    assert_eq!(vy, 6, "cursor Y after 2 wrapped lines of 3 rows each");
}

#[test]
fn visual_cursor_y_after_o_between_wrapped_lines() {
    // Insert new line between two long wrapped lines
    let content = format!("{}\n{}", "A".repeat(40), "B".repeat(40));
    let mut te = TestEditor::new(&content, 20, 20);
    te.set_opt("wrap");
    te.set_opt("nonu");
    te.feed("o"); // open below line 0 → new line 1
    let (_, vy) = te.visual_cursor().expect("cursor visible");
    assert_eq!(te.line(), 1);
    // Line 0 wraps to 2 rows, new line at visual row 2
    assert_eq!(vy, 2, "new line appears after wrapped line");
}

#[test]
fn visual_cursor_y_no_wrap_scrolled() {
    // 30 lines, viewport 10 high, scroll past top
    let lines: Vec<String> = (0..30).map(|i| format!("line{i:02}")).collect();
    let content = lines.join("\n");
    let mut te = TestEditor::new(&content, 40, 10);
    te.set_opt("nowrap");
    te.set_opt("nonu");
    te.feed("15G"); // go to line 15 (0-indexed: 14)
    let (_, vy) = te.visual_cursor().expect("cursor visible");
    let scroll = te.ev.editor().viewport_scroll();
    let expected_y = (14 - scroll) as u16;
    assert_eq!(vy, expected_y, "cursor Y = line - scroll when no wrap");
}

#[test]
fn visual_cursor_y_wrap_scrolled() {
    // Long lines + scroll — cursor Y must account for wrapped lines in viewport
    let lines: Vec<String> = (0..10).map(|i| format!("{}", (b'a' + i) as char).repeat(40)).collect();
    let content = lines.join("\n");
    let mut te = TestEditor::new(&content, 20, 10);
    te.set_opt("wrap");
    te.set_opt("nonu");
    te.feed("5G"); // go to line 5 (0-indexed: 4)
                   // With 10 lines × 2 visual rows each, line 4 starts at visual row 8 from top.
                   // Scroll should have adjusted. Cursor should be visible within viewport.
    if let Some((_, vy)) = te.visual_cursor() {
        assert!(vy < 10, "cursor within viewport height, got vy={vy}");
    }
}

#[test]
fn wrap_nu_o_sequence() {
    let long = "x".repeat(40);
    let content = format!("{long}\nshort\n{long}");
    let mut te = TestEditor::new(&content, 30, 20);
    te.set_opt("wrap");
    te.set_opt("nu");
    te.feed("jo"); // line 1 (short), open below
    assert_eq!(te.line(), 2);
    assert_eq!(te.line_text(2), "");
    assert_eq!(te.mode(), EditorMode::Insert);
}

#[test]
fn wrap_nu_dd_paste_cycle() {
    let content = "aaa\nbbb\nccc\nddd";
    let mut te = TestEditor::new(content, 30, 20);
    te.set_opt("wrap");
    te.set_opt("nu");
    te.feed("jdd"); // delete bbb
    assert_eq!(te.content().trim(), "aaa\nccc\nddd");
    te.feed("p"); // paste below
    assert_eq!(te.line_text(2), "bbb");
}
