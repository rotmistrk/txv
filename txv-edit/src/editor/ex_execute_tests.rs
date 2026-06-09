#![cfg(test)]
//! Ex command execution tests — verify buffer modifications.

use crate::editor::command::Command;
use crate::editor::Editor;

#[test]
fn ex_sort_whole_file() {
    let mut ed = Editor::from_text("cherry\napple\nbanana");
    ed.set_viewport_height(20);
    ed.execute(Command::ExCommand("%!sort".to_string()));
    assert_eq!(ed.buf().content(), "apple\nbanana\ncherry\n");
}

#[test]
fn ex_range_filter_nl() {
    let mut ed = Editor::from_text("aaa\nbbb\nccc\nddd");
    ed.set_viewport_height(20);
    ed.execute(Command::ExCommand("2,3!nl".to_string()));
    let c = ed.buf().content();
    assert!(c.contains("1\t"), "nl should number lines: {c}");
}

#[test]
fn ex_yank_range() {
    let text = (1..=15).map(|i| format!("line {i}")).collect::<Vec<_>>().join("\n");
    let mut ed = Editor::from_text(&text);
    ed.set_viewport_height(20);
    ed.execute(Command::ExCommand("4,10y".to_string()));
    let reg = ed.register();
    assert!(reg.starts_with("line 4\n"), "register starts with line 4: {reg}");
    assert!(reg.contains("line 10\n"), "register contains line 10");
}

#[test]
fn substitute_single() {
    let mut ed = Editor::from_text("hello world hello");
    ed.set_viewport_height(10);
    ed.execute(Command::ExCommand("s/hello/bye/".to_string()));
    assert_eq!(ed.buf().content(), "bye world hello");
}

#[test]
fn substitute_global_on_line() {
    let mut ed = Editor::from_text("hello world hello");
    ed.set_viewport_height(10);
    ed.execute(Command::ExCommand("s/hello/bye/g".to_string()));
    assert_eq!(ed.buf().content(), "bye world bye");
}

#[test]
fn substitute_percent_range() {
    let mut ed = Editor::from_text("aaa\nbbb\naaa\nbbb");
    ed.set_viewport_height(10);
    ed.execute(Command::ExCommand("%s/aaa/zzz/g".to_string()));
    assert_eq!(ed.buf().content(), "zzz\nbbb\nzzz\nbbb");
}

#[test]
fn substitute_with_regex_groups() {
    let mut ed = Editor::from_text("foo123 bar456");
    ed.set_viewport_height(10);
    ed.execute(Command::ExCommand("s/[0-9]+/NUM/g".to_string()));
    assert_eq!(ed.buf().content(), "fooNUM barNUM");
}

#[test]
fn substitute_with_capture_group() {
    let mut ed = Editor::from_text("hello-world");
    ed.set_viewport_height(10);
    ed.execute(Command::ExCommand("s/(\\w+)-(\\w+)/$2-$1/".to_string()));
    assert_eq!(ed.buf().content(), "world-hello");
}

#[test]
fn search_forward() {
    let mut ed = Editor::from_text("alpha beta gamma beta delta");
    ed.set_viewport_height(10);
    ed.execute(Command::SearchForward("beta".into()));
    assert_eq!(ed.cursor_col(), 6);
}

#[test]
fn search_backward() {
    let mut ed = Editor::from_text("alpha beta gamma beta delta");
    ed.set_viewport_height(10);
    ed.set_cursor_col(26);
    ed.execute(Command::SearchBackward("beta".into()));
    assert_eq!(ed.cursor_col(), 17);
}

#[test]
fn search_next_wraps() {
    let mut ed = Editor::from_text("foo bar foo baz");
    ed.set_viewport_height(10);
    ed.execute(Command::SearchForward("foo".into()));
    ed.execute(Command::SearchNext);
    // Should wrap to beginning
    assert_eq!(ed.cursor_col(), 0);
}

#[test]
fn ex_delete_range() {
    let mut ed = Editor::from_text("one\ntwo\nthree\nfour");
    ed.set_viewport_height(10);
    ed.execute(Command::ExCommand("2,3d".to_string()));
    assert_eq!(ed.buf().content(), "one\nfour");
}
