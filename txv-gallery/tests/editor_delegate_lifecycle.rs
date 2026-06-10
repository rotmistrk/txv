//! Tests for EditorViewDelegate lifecycle hooks (cursor, mode).

use txv_core::prelude::*;
use txv_edit::editor::keymap::EditorMode;
use txv_edit::editor::Editor;
use txv_edit::view::delegate::EditorViewDelegate;
use txv_edit::view::EditorView;

/// Combined lifecycle delegate tracking cursor moves and mode transitions.
struct LifecycleDelegate {
    moved: bool,
    transitions: Vec<(EditorMode, EditorMode)>,
}

impl LifecycleDelegate {
    fn new() -> Self {
        Self {
            moved: false,
            transitions: Vec::new(),
        }
    }
}

impl EditorViewDelegate for LifecycleDelegate {
    fn on_cursor_moved(&mut self, _editor: &Editor) {
        self.moved = true;
    }
    fn on_mode_changed(&mut self, old: EditorMode, new: EditorMode, _editor: &Editor) {
        self.transitions.push((old, new));
    }
}

#[test]
fn delegate_on_cursor_moved() {
    let mut ev = EditorView::with_delegate(LifecycleDelegate::new());
    ev.set_content("line1\nline2\nline3", "");
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    ev.handle(&Event::Key(KeyEvent::new(KeyCode::Char('j'), KeyMod::NONE)));
    assert!(ev.delegate().moved);
}

#[test]
fn delegate_on_mode_changed() {
    let mut ev = EditorView::with_delegate(LifecycleDelegate::new());
    ev.set_content("hello", "");
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    ev.handle(&Event::Key(KeyEvent::new(KeyCode::Char('i'), KeyMod::NONE)));
    ev.handle(&Event::Key(KeyEvent::new(KeyCode::Esc, KeyMod::NONE)));
    let t = &ev.delegate().transitions;
    assert_eq!(t.len(), 2);
    assert_eq!(t[0], (EditorMode::Normal, EditorMode::Insert));
    assert_eq!(t[1], (EditorMode::Insert, EditorMode::Normal));
}
