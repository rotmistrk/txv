use super::*;

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyMod::default(),
    }
}

#[test]
fn insert_and_commit() {
    let mut ed = InlineEditor::new(0, "");
    assert_eq!(ed.handle_key(&key(KeyCode::Char('h'))), InlineEditResult::Continue);
    assert_eq!(ed.handle_key(&key(KeyCode::Char('i'))), InlineEditResult::Continue);
    assert_eq!(ed.buffer, "hi");
    assert_eq!(ed.cursor, 2);
    assert_eq!(
        ed.handle_key(&key(KeyCode::Enter)),
        InlineEditResult::Commit("hi".to_owned())
    );
}

#[test]
fn cancel() {
    let mut ed = InlineEditor::new(0, "text");
    assert_eq!(ed.handle_key(&key(KeyCode::Esc)), InlineEditResult::Cancel);
}

#[test]
fn backspace_and_delete() {
    let mut ed = InlineEditor::new(0, "abc");
    ed.handle_key(&key(KeyCode::Backspace));
    assert_eq!(ed.buffer, "ab");
    assert_eq!(ed.cursor, 2);
    ed.handle_key(&key(KeyCode::Home));
    ed.handle_key(&key(KeyCode::Delete));
    assert_eq!(ed.buffer, "b");
    assert_eq!(ed.cursor, 0);
}

#[test]
fn navigation() {
    let mut ed = InlineEditor::new(0, "hello");
    ed.handle_key(&key(KeyCode::Home));
    assert_eq!(ed.cursor, 0);
    ed.handle_key(&key(KeyCode::Right));
    assert_eq!(ed.cursor, 1);
    ed.handle_key(&key(KeyCode::End));
    assert_eq!(ed.cursor, 5);
    ed.handle_key(&key(KeyCode::Left));
    assert_eq!(ed.cursor, 4);
}

#[test]
fn tab_completion() {
    let mut ed = InlineEditor::new(0, "");
    let candidates = vec!["alpha".to_owned(), "beta".to_owned(), "gamma".to_owned()];
    ed.apply_completion(&candidates, 1);
    assert_eq!(ed.buffer, "alpha");
    ed.apply_completion(&candidates, 1);
    assert_eq!(ed.buffer, "beta");
    ed.apply_completion(&candidates, 1);
    assert_eq!(ed.buffer, "gamma");
    ed.apply_completion(&candidates, 1);
    assert_eq!(ed.buffer, "alpha");
}

fn shift_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyMod {
            shift: true,
            ..KeyMod::default()
        },
    }
}

#[test]
fn new_selected_selects_all() {
    let ed = InlineEditor::new_selected(0, "hello");
    assert_eq!(ed.anchor, Some(0));
    assert_eq!(ed.cursor, 5);
    assert_eq!(ed.selection_range(), Some((0, 5)));
}

#[test]
fn type_replaces_selection() {
    let mut ed = InlineEditor::new_selected(0, "old");
    ed.handle_key(&key(KeyCode::Char('n')));
    assert_eq!(ed.buffer, "n");
    assert_eq!(ed.cursor, 1);
    assert_eq!(ed.anchor, None);
}

#[test]
fn shift_arrow_extends_selection() {
    let mut ed = InlineEditor::new(0, "abcde");
    ed.handle_key(&key(KeyCode::Home));
    ed.handle_key(&shift_key(KeyCode::Right));
    ed.handle_key(&shift_key(KeyCode::Right));
    assert_eq!(ed.selection_range(), Some((0, 2)));
    ed.handle_key(&key(KeyCode::Right));
    assert_eq!(ed.anchor, None);
}

#[test]
fn backspace_deletes_selection() {
    let mut ed = InlineEditor::new_selected(0, "hello");
    ed.handle_key(&key(KeyCode::Backspace));
    assert_eq!(ed.buffer, "");
    assert_eq!(ed.cursor, 0);
}

#[test]
fn tab_commits() {
    let mut ed = InlineEditor::new(0, "text");
    assert_eq!(
        ed.handle_key(&key(KeyCode::Tab)),
        InlineEditResult::Commit("text".to_owned())
    );
}

#[test]
fn shift_home_selects_to_start() {
    let mut ed = InlineEditor::new(0, "hello");
    ed.handle_key(&shift_key(KeyCode::Home));
    assert_eq!(ed.selection_range(), Some((0, 5)));
    assert_eq!(ed.cursor, 0);
    assert_eq!(ed.anchor, Some(5));
}

#[test]
fn shift_end_selects_to_end() {
    let mut ed = InlineEditor::new(0, "hello");
    ed.handle_key(&key(KeyCode::Home));
    ed.handle_key(&shift_key(KeyCode::End));
    assert_eq!(ed.selection_range(), Some((0, 5)));
    assert_eq!(ed.cursor, 5);
    assert_eq!(ed.anchor, Some(0));
}

#[test]
fn delete_key_removes_selection() {
    let mut ed = InlineEditor::new(0, "abcde");
    ed.handle_key(&key(KeyCode::Home));
    ed.handle_key(&shift_key(KeyCode::Right));
    ed.handle_key(&shift_key(KeyCode::Right));
    ed.handle_key(&shift_key(KeyCode::Right));
    ed.handle_key(&key(KeyCode::Delete));
    assert_eq!(ed.buffer, "de");
    assert_eq!(ed.cursor, 0);
}

#[test]
fn type_mid_selection_replaces() {
    let mut ed = InlineEditor::new(0, "abcde");
    ed.cursor = 1;
    ed.anchor = Some(4);
    ed.handle_key(&key(KeyCode::Char('X')));
    assert_eq!(ed.buffer, "aXe");
    assert_eq!(ed.cursor, 2);
}

#[test]
fn selection_range_none_without_anchor() {
    let ed = InlineEditor::new(0, "text");
    assert_eq!(ed.selection_range(), None);
}

#[test]
fn nav_after_selection_clears_anchor() {
    let mut ed = InlineEditor::new_selected(0, "abc");
    ed.handle_key(&key(KeyCode::Left));
    assert_eq!(ed.anchor, None);
    assert_eq!(ed.cursor, 2);
}
