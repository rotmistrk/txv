//! Editor vi-mode integration tests using EditorView + MockBackend.

use txv_core::prelude::*;
use txv_edit::editor::command::Command;
use txv_edit::editor::keymap::EditorMode;
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

/// Helper: feed a special key.
fn feed_key(ev: &mut EditorView, code: KeyCode, mods: KeyMod) {
    ev.handle(&Event::Key(KeyEvent::new(code, mods)));
}

/// Helper: get buffer content from editor.
fn content(ev: &EditorView) -> String {
    ev.content()
}

// ===== Basic motions =====

#[test]
fn goto_line_22g() {
    let text = (1..=30).map(|i| format!("line {i}")).collect::<Vec<_>>().join("\n");
    let mut ev = editor_with(&text, 40, 10);
    feed(&mut ev, "22G");
    assert_eq!(ev.editor().cursor_line(), 21); // 0-indexed
}

#[test]
fn move_to_end_of_file() {
    let mut ev = editor_with("a\nb\nc\nd\ne", 40, 10);
    feed(&mut ev, "G");
    assert_eq!(ev.editor().cursor_line(), 4);
}

#[test]
fn move_to_start_of_file() {
    let mut ev = editor_with("a\nb\nc", 40, 10);
    feed(&mut ev, "Ggg");
    assert_eq!(ev.editor().cursor_line(), 0);
}

// ===== Yank and Paste =====

#[test]
fn yank_line_and_paste() {
    let mut ev = editor_with("alpha\nbeta\ngamma", 40, 10);
    feed(&mut ev, "yy"); // yank line 1
    feed(&mut ev, "j"); // move to line 2
    feed(&mut ev, "p"); // paste below
    assert_eq!(content(&ev), "alpha\nbeta\nalpha\ngamma");
}

#[test]
fn yank_range_4_plus_6y() {
    // Ex command "4,+6y" means yank lines 4..10 (1-indexed)
    let text = (1..=15).map(|i| format!("line {i}")).collect::<Vec<_>>().join("\n");
    let mut ed = Editor::from_text(&text);
    ed.set_viewport_height(20);
    let cmd = Command::ExCommand("4,10y".to_string());
    ed.execute(cmd);
    // Verify register contains lines 4-10
    let reg = ed.register();
    assert!(reg.starts_with("line 4\n"), "register should start with line 4: {reg}");
    assert!(reg.contains("line 10\n"), "register should contain line 10");
}

// ===== Visual modes =====

#[test]
fn visual_line_select_and_yank() {
    let mut ev = editor_with("one\ntwo\nthree\nfour", 40, 10);
    feed(&mut ev, "V"); // enter visual line
    assert_eq!(ev.editor().mode(), EditorMode::VisualLine);
    feed(&mut ev, "j"); // extend to line 2
    feed(&mut ev, "y"); // yank
    assert_eq!(ev.editor().mode(), EditorMode::Normal);
    assert!(ev.editor().register().contains("one\ntwo\n"));
}

#[test]
fn visual_char_select_and_yank() {
    let mut ev = editor_with("hello world", 40, 10);
    feed(&mut ev, "v"); // enter visual char
    assert_eq!(ev.editor().mode(), EditorMode::Visual);
    feed(&mut ev, "llll"); // select "hello"
    feed(&mut ev, "y");
    assert_eq!(ev.editor().register(), "hello");
}

#[test]
fn visual_block_select() {
    let mut ev = editor_with("abcd\nefgh\nijkl", 40, 10);
    // Ctrl-V to enter block visual
    feed_key(&mut ev, KeyCode::Char('v'), KeyMod::CTRL);
    assert_eq!(ev.editor().mode(), EditorMode::VisualBlock);
    feed(&mut ev, "jl"); // 2 rows, 2 cols
    feed(&mut ev, "y");
    // Block yank: "ab\nef"
    assert!(ev.editor().register_block());
    assert_eq!(ev.editor().register(), "ab\nef");
}

// ===== Paste (P and p) =====

#[test]
fn paste_before_with_capital_p() {
    let mut ev = editor_with("alpha\nbeta\ngamma", 40, 10);
    feed(&mut ev, "jyy"); // go to beta, yank it
    feed(&mut ev, "gg"); // go to top
    feed(&mut ev, "P"); // paste before (above line 0)
    assert_eq!(content(&ev), "beta\nalpha\nbeta\ngamma");
}

// ===== Delete and Change =====

#[test]
fn delete_line_dd() {
    let mut ev = editor_with("one\ntwo\nthree", 40, 10);
    feed(&mut ev, "jdd"); // delete "two"
    assert_eq!(content(&ev), "one\nthree");
}

#[test]
fn change_word_cw() {
    let mut ev = editor_with("hello world", 40, 10);
    feed(&mut ev, "cw");
    feed(&mut ev, "goodbye\x1b");
    // cw changes to end of word (includes trailing space in vi)
    assert_eq!(content(&ev), "goodbyeworld");
}

// ===== Ex commands =====
// Ex commands bypass the keymap — they're submitted directly to Editor::execute.

#[test]
fn ex_sort_whole_file() {
    let mut ed = Editor::from_text("cherry\napple\nbanana");
    ed.set_viewport_height(20);
    let cmd = Command::ExCommand("%!sort".to_string());
    ed.execute(cmd);
    assert_eq!(ed.buf().content(), "apple\nbanana\ncherry\n");
}

#[test]
fn ex_range_filter_nl() {
    let mut ed = Editor::from_text("aaa\nbbb\nccc\nddd");
    ed.set_viewport_height(20);
    // Filter lines 2-3 (0-indexed: 1-2) through nl
    let cmd = Command::ExCommand("2,3!nl".to_string());
    ed.execute(cmd);
    let c = ed.buf().content();
    assert!(c.contains("1\t"), "nl should number lines: {c}");
}

// ===== Undo/Redo =====

#[test]
fn undo_restores_content() {
    let mut ev = editor_with("original", 40, 10);
    feed(&mut ev, "dd");
    assert_eq!(content(&ev), "");
    feed(&mut ev, "u");
    assert_eq!(content(&ev), "original");
}

#[test]
fn redo_after_undo() {
    let mut ev = editor_with("original", 40, 10);
    feed(&mut ev, "dd");
    feed(&mut ev, "u");
    feed_key(&mut ev, KeyCode::Char('r'), KeyMod::CTRL);
    assert_eq!(content(&ev), "");
}

// ===== Insert mode =====

#[test]
fn insert_mode_typing() {
    let mut ev = editor_with("", 40, 10);
    feed(&mut ev, "i");
    assert_eq!(ev.editor().mode(), EditorMode::Insert);
    feed(&mut ev, "hello\x1b");
    assert_eq!(content(&ev), "hello");
    assert_eq!(ev.editor().mode(), EditorMode::Normal);
}

// ===== Marks =====

#[test]
fn set_and_jump_to_mark() {
    let mut ev = editor_with("line1\nline2\nline3", 40, 10);
    feed(&mut ev, "j"); // go to line 2
    feed(&mut ev, "ma"); // set mark 'a'
    feed(&mut ev, "gg"); // go to top
    feed(&mut ev, "'a"); // jump to mark a
    assert_eq!(ev.editor().cursor_line(), 1);
}

// ===== Match bracket =====

#[test]
fn match_bracket_percent() {
    let mut ev = editor_with("(hello)", 40, 10);
    feed(&mut ev, "%");
    assert_eq!(ev.editor().cursor_col(), 6); // closing paren
    feed(&mut ev, "%");
    assert_eq!(ev.editor().cursor_col(), 0); // back to opening
}
