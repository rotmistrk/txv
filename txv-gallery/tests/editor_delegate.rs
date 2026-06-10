//! Tests for EditorViewDelegate hooks.

use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use txv_core::event::CommandId;
use txv_core::prelude::*;
use txv_edit::editor::keymap::EditorMode;
use txv_edit::editor::Editor;
use txv_edit::view::delegate::EditorViewDelegate;
use txv_edit::view::EditorView;

// --- Tick delegate ---

struct TickDelegate {
    count: Arc<AtomicU64>,
}
impl EditorViewDelegate for TickDelegate {
    fn on_tick(&mut self, _editor: &mut Editor, tick: u64) -> HandleResult {
        self.count.store(tick, Ordering::Relaxed);
        HandleResult::Ignored
    }
}

#[test]
fn delegate_on_tick_called() {
    let count = Arc::new(AtomicU64::new(0));
    let mut ev = EditorView::with_delegate(TickDelegate { count: count.clone() });
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    ev.handle(&Event::Tick);
    ev.handle(&Event::Tick);
    assert_eq!(count.load(Ordering::Relaxed), 2);
}

// --- Command delegate ---

struct CmdDelegate {
    last_cmd: Option<CommandId>,
}
impl EditorViewDelegate for CmdDelegate {
    fn on_command(&mut self, id: CommandId, _data: &Option<Box<dyn Any + Send>>, _editor: &mut Editor) -> HandleResult {
        self.last_cmd = Some(id);
        HandleResult::Consumed
    }
}

#[test]
fn delegate_on_command_intercepts() {
    let mut ev = EditorView::with_delegate(CmdDelegate { last_cmd: None });
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    let cmd = Event::Command {
        id: 999,
        data: None,
        broadcast: false,
    };
    let result = ev.handle(&cmd);
    assert_eq!(result, HandleResult::Consumed);
    assert_eq!(ev.delegate().last_cmd, Some(999));
}

// --- Key pre delegate ---

struct KeyPreDelegate {
    intercepted: bool,
}
impl EditorViewDelegate for KeyPreDelegate {
    fn on_key_pre(&mut self, key: &KeyEvent, _editor: &mut Editor) -> Option<HandleResult> {
        if key.modifiers().ctrl() && key.code() == KeyCode::Char('n') {
            self.intercepted = true;
            return Some(HandleResult::Consumed);
        }
        None
    }
}

#[test]
fn delegate_on_key_pre_intercepts() {
    let mut ev = EditorView::with_delegate(KeyPreDelegate { intercepted: false });
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    // Ctrl-N should be intercepted
    let key = Event::Key(KeyEvent::new(KeyCode::Char('n'), KeyMod::CTRL));
    ev.handle(&key);
    assert!(ev.delegate().intercepted);
}

#[test]
fn delegate_on_key_pre_passes_through() {
    let mut ev = EditorView::with_delegate(KeyPreDelegate { intercepted: false });
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    // Plain 'j' should pass through (cursor moves)
    let key = Event::Key(KeyEvent::new(KeyCode::Char('j'), KeyMod::NONE));
    ev.handle(&key);
    assert!(!ev.delegate().intercepted);
}

// --- Cursor moved delegate ---

struct CursorDelegate {
    moved: bool,
}
impl EditorViewDelegate for CursorDelegate {
    fn on_cursor_moved(&mut self, _editor: &Editor) {
        self.moved = true;
    }
}

#[test]
fn delegate_on_cursor_moved() {
    let mut ev = EditorView::with_delegate(CursorDelegate { moved: false });
    ev.set_content("line1\nline2\nline3", "");
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    // Press 'j' to move down
    let key = Event::Key(KeyEvent::new(KeyCode::Char('j'), KeyMod::NONE));
    ev.handle(&key);
    assert!(ev.delegate().moved);
}

// --- Mode changed delegate ---

struct ModeDelegate {
    transitions: Vec<(EditorMode, EditorMode)>,
}
impl EditorViewDelegate for ModeDelegate {
    fn on_mode_changed(&mut self, old: EditorMode, new: EditorMode, _editor: &Editor) {
        self.transitions.push((old, new));
    }
}

#[test]
fn delegate_on_mode_changed() {
    let mut ev = EditorView::with_delegate(ModeDelegate {
        transitions: Vec::new(),
    });
    ev.set_content("hello", "");
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    // 'i' enters insert mode
    ev.handle(&Event::Key(KeyEvent::new(KeyCode::Char('i'), KeyMod::NONE)));
    // Esc returns to normal
    ev.handle(&Event::Key(KeyEvent::new(KeyCode::Esc, KeyMod::NONE)));
    let t = &ev.delegate().transitions;
    assert_eq!(t.len(), 2);
    assert_eq!(t[0], (EditorMode::Normal, EditorMode::Insert));
    assert_eq!(t[1], (EditorMode::Insert, EditorMode::Normal));
}

// --- Title delegate ---

struct TitleDelegate;
impl EditorViewDelegate for TitleDelegate {
    fn title(&self, _editor: &Editor) -> Option<&str> {
        Some("Custom Title")
    }
}

#[test]
fn delegate_title_override() {
    let ev = EditorView::with_delegate(TitleDelegate);
    assert_eq!(ev.title(), "Custom Title");
}

// --- Gutter sign delegate ---

struct GutterSignDelegate;
impl EditorViewDelegate for GutterSignDelegate {
    fn gutter_sign(&self, line: usize) -> Option<(char, Style)> {
        if line == 1 {
            Some(('!', Style::default()))
        } else {
            None
        }
    }
    fn extra_gutter_width(&self) -> u16 {
        1
    }
}

#[test]
fn delegate_gutter_sign() {
    let mut ev = EditorView::with_delegate(GutterSignDelegate);
    ev.set_content("line1\nline2\nline3", "");
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    ev.render();
    // Line 2 (index 1) should have '!' in the gutter sign column (col 0)
    let ch = ev.buffer().cell(0, 1).ch();
    assert_eq!(ch, '!', "gutter sign at line 2");
}

// --- Highlight ranges delegate ---

use txv_edit::view::delegate::HighlightRange;

struct HighlightDelegate;
impl EditorViewDelegate for HighlightDelegate {
    fn highlight_ranges(&self, line: usize) -> &[HighlightRange] {
        // Static trick: leak a vec for test purposes
        static RANGES: std::sync::LazyLock<Vec<HighlightRange>> = std::sync::LazyLock::new(|| {
            vec![HighlightRange {
                col_start: 0,
                col_end: 3,
                style: Style::new(Color::Reset, Color::Rgb(255, 255, 0)),
            }]
        });
        if line == 0 {
            &RANGES
        } else {
            &[]
        }
    }
}

#[test]
fn delegate_highlight_ranges() {
    let mut ev = EditorView::with_delegate(HighlightDelegate);
    ev.set_content("hello world", "");
    ev.editor_mut().options_mut().set_number(false);
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    ev.render();
    // Col 1 (not cursor position) should have yellow background
    let cell = ev.buffer().cell(1, 0);
    assert_eq!(cell.style().bg(), Color::Rgb(255, 255, 0), "highlight bg at col 1");
}

// --- Software cursor delegate ---

use txv_edit::view::delegate::CursorRender;

struct SoftwareCursorDelegate;
impl EditorViewDelegate for SoftwareCursorDelegate {
    fn cursor_render(&self, _mode: EditorMode) -> CursorRender {
        CursorRender::Software(Style::new(Color::Rgb(0, 0, 0), Color::Rgb(255, 255, 255)))
    }
}

#[test]
fn delegate_software_cursor_no_hardware() {
    let mut ev = EditorView::with_delegate(SoftwareCursorDelegate);
    ev.set_content("hello", "");
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    ev.render();
    // Hardware cursor should NOT be reported
    assert!(ev.cursor().is_none(), "no hardware cursor with Software render");
}

#[test]
fn delegate_software_cursor_style() {
    let mut ev = EditorView::with_delegate(SoftwareCursorDelegate);
    ev.set_content("hello", "");
    ev.editor_mut().options_mut().set_number(false);
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    ev.render();
    // Cursor at (0,0) — no gutter, cell should have delegate's cursor style
    let cell = ev.buffer().cell(0, 0);
    assert_eq!(cell.style().fg(), Color::Rgb(0, 0, 0), "software cursor fg");
    assert_eq!(cell.style().bg(), Color::Rgb(255, 255, 255), "software cursor bg");
}
