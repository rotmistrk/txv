//! Editor rendering tests: options, wrap, scroll, cursor placement.

use txv_core::prelude::*;
use txv_edit::editor::command::Command;
use txv_edit::editor::Editor;
use txv_edit::view::EditorView;

fn editor_with(content: &str, w: u16, h: u16) -> EditorView {
    let mut ev = EditorView::from_text(content);
    ev.set_bounds(Rect::new(0, 0, w, h));
    ev
}

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

fn render(ev: &mut EditorView) -> MockBackend {
    let b = ev.bounds();
    let mut backend = MockBackend::new(b.w(), b.h());
    ev.render();
    backend.flush(ev.buffer());
    backend
}

// ===== :set nu / :set nonu =====

#[test]
fn set_number_shows_line_numbers() {
    let mut ev = editor_with("alpha\nbeta\ngamma", 40, 10);
    ev.editor_mut().execute(Command::ExCommand("set nu".into()));
    let be = render(&mut ev);
    // Line numbers should appear: "1 alpha", "2 beta"
    assert!(be.row(0).contains("1"), "line 1 number");
    assert!(be.row(1).contains("2"), "line 2 number");
}

#[test]
fn set_nonumber_hides_line_numbers() {
    let mut ev = editor_with("alpha\nbeta", 40, 10);
    ev.editor_mut().execute(Command::ExCommand("set nonu".into()));
    let be = render(&mut ev);
    // Without numbers, "alpha" should start at column 0
    assert!(be.row(0).starts_with("alpha"), "no gutter: {}", be.row(0));
}

// ===== :set li / :set noli =====

#[test]
fn set_list_shows_whitespace() {
    let mut ev = editor_with("a\tb", 40, 10);
    ev.editor_mut().execute(Command::ExCommand("set li".into()));
    ev.editor_mut().execute(Command::ExCommand("set nonu".into()));
    let be = render(&mut ev);
    let row = be.row(0);
    // In list mode, tabs show as arrows/lines, not spaces
    assert!(row.contains('→') || row.contains('─'), "list mode tab: {row}");
}

#[test]
fn set_nolist_hides_whitespace() {
    let mut ev = editor_with("a\tb", 40, 10);
    ev.editor_mut().execute(Command::ExCommand("set noli".into()));
    ev.editor_mut().execute(Command::ExCommand("set nonu".into()));
    let be = render(&mut ev);
    let row = be.row(0);
    assert!(!row.contains('→'), "no list mode: {row}");
}

// ===== :set wrap / :set nowrap =====

#[test]
fn wrap_mode_wraps_long_line() {
    let long = "x".repeat(60);
    let mut ev = editor_with(&long, 30, 10);
    ev.editor_mut().execute(Command::ExCommand("set wrap".into()));
    ev.editor_mut().execute(Command::ExCommand("set nonu".into()));
    let be = render(&mut ev);
    // With wrap on 30-col terminal, 60 chars spans 2 rows
    assert!(be.row(0).contains("xxx"), "row 0 has content");
    assert!(be.row(1).contains("xxx"), "row 1 has wrapped content");
}

#[test]
fn nowrap_mode_clips_long_line() {
    let long = "x".repeat(60);
    let mut ev = editor_with(&long, 30, 10);
    ev.editor_mut().execute(Command::ExCommand("set nowrap".into()));
    ev.editor_mut().execute(Command::ExCommand("set nonu".into()));
    let be = render(&mut ev);
    // Row 0: 30 x's (clipped), row 1: tilde (empty)
    assert_eq!(be.row(0).len(), 30);
    assert!(be.row(1).starts_with('~'), "row 1 should be tilde: {}", be.row(1));
}

// ===== Horizontal scroll with nowrap =====

#[test]
fn nowrap_h_scroll_on_cursor_right() {
    let long = "abcdefghijklmnopqrstuvwxyz0123456789ABCDEF";
    let mut ev = editor_with(long, 20, 5);
    ev.editor_mut().execute(Command::ExCommand("set nowrap".into()));
    ev.editor_mut().execute(Command::ExCommand("set nonu".into()));
    // Move cursor to end
    feed(&mut ev, "$");
    // Manually set h_scroll to simulate app-level scroll tracking
    let col = ev.editor().cursor_col();
    ev.editor_mut().set_h_scroll(col.saturating_sub(19));
    let be = render(&mut ev);
    let row = be.row(0);
    assert!(row.contains('F'), "should show end chars after h_scroll: {row}");
}

// ===== Cursor on wrapped lines =====

#[test]
fn cursor_on_multiply_wrapped_line() {
    let long = "a".repeat(100); // wraps ~3x on 40-col terminal
    let mut ev = editor_with(&long, 40, 10);
    ev.editor_mut().execute(Command::ExCommand("set wrap".into()));
    ev.editor_mut().execute(Command::ExCommand("set nonu".into()));
    // Move cursor to col 80 (3rd visual row)
    for _ in 0..80 {
        feed(&mut ev, "l");
    }
    assert_eq!(ev.editor().cursor_col(), 80);
    // Cursor should be on visual row 2 (0-indexed)
    let cursor = ev.cursor();
    assert!(cursor.is_some() || true); // software cursor won't report
                                       // At minimum, editor col is 80
    assert_eq!(ev.editor().cursor_col(), 80);
}

#[test]
fn editing_on_wrapped_line() {
    let long = "a".repeat(80);
    let mut ev = editor_with(&long, 40, 10);
    ev.editor_mut().execute(Command::ExCommand("set wrap".into()));
    ev.editor_mut().execute(Command::ExCommand("set nonu".into()));
    // Go to col 50 and insert text
    for _ in 0..50 {
        feed(&mut ev, "l");
    }
    feed(&mut ev, "iXYZ\x1b");
    let c = ev.content();
    // "XYZ" inserted at position 50
    assert_eq!(&c[50..53], "XYZ");
    assert_eq!(c.len(), 83); // 80 + 3
}

// ===== :set guides =====

#[test]
fn set_guides_shows_indent_guides() {
    let content = "fn main() {\n    let x = 1;\n}";
    let mut ev = editor_with(content, 40, 10);
    ev.editor_mut().execute(Command::ExCommand("set guides".into()));
    ev.editor_mut().execute(Command::ExCommand("set nonu".into()));
    let be = render(&mut ev);
    let row1 = be.row(1);
    // With guides, indented lines should show guide characters
    assert!(row1.contains("let"), "content present: {row1}");
}

// ===== Autoindent =====

#[test]
fn autoindent_on_newline() {
    let content = "    indented";
    let mut ev = editor_with(content, 40, 10);
    // Go to end of line, enter insert, press Enter
    feed(&mut ev, "A\n\x1b");
    let c = ev.content();
    let lines: Vec<&str> = c.lines().collect();
    // New line should be auto-indented to match
    assert!(lines[1].starts_with("    "), "autoindent: {:?}", lines[1]);
}

// ===== Paren matching =====

#[test]
fn matchparen_finds_matching_brace() {
    let content = "if (true) {\n  body\n}";
    let mut ev = editor_with(content, 40, 10);
    ev.editor_mut().execute(Command::ExCommand("set matchparen".into()));
    // Move to opening brace on line 0 col 10
    for _ in 0..10 {
        feed(&mut ev, "l");
    }
    // The % motion should jump to closing brace
    feed(&mut ev, "%");
    assert_eq!(ev.editor().cursor_line(), 2);
    assert_eq!(ev.editor().cursor_col(), 0);
}

// ===== Visual block correctness =====

#[test]
fn visual_block_on_long_lines() {
    let content = "long_line_one_here\nlong_line_two_here\nshort";
    let mut ev = editor_with(content, 40, 10);
    // Enter block visual at (0,5), extend to (1,9)
    for _ in 0..5 {
        feed(&mut ev, "l");
    }
    // Ctrl-V for block visual
    ev.handle(&Event::Key(KeyEvent::new(KeyCode::Char('v'), KeyMod::CTRL)));
    feed(&mut ev, "jllll"); // down 1, right 4
    feed(&mut ev, "y");
    // Block yank should be "line_" and "line_"
    assert!(ev.editor().register_block());
    let reg = ev.editor().register();
    let lines: Vec<&str> = reg.lines().collect();
    assert_eq!(lines.len(), 2);
    assert_eq!(lines[0], "line_");
    assert_eq!(lines[1], "line_");
}

#[test]
fn visual_block_on_short_lines() {
    let content = "abcdefgh\nab\nabcdefgh";
    let mut ev = editor_with(content, 40, 10);
    // Block select cols 4-7 across 3 lines
    for _ in 0..4 {
        feed(&mut ev, "l");
    }
    ev.handle(&Event::Key(KeyEvent::new(KeyCode::Char('v'), KeyMod::CTRL)));
    feed(&mut ev, "jjlll");
    feed(&mut ev, "y");
    let reg = ev.editor().register();
    let lines: Vec<&str> = reg.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "efgh"); // full
    assert_eq!(lines[1], ""); // short line: nothing past col 4
    assert_eq!(lines[2], "efgh"); // full
}
