//! Tests for EditorViewDelegate event interception hooks.

use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use txv_core::event::CommandId;
use txv_core::prelude::*;
use txv_edit::editor::Editor;
use txv_edit::view::delegate::EditorViewDelegate;
use txv_edit::view::EditorView;

/// Multi-purpose test delegate tracking ticks, commands, and key intercepts.
struct TestDelegate {
    tick_count: Arc<AtomicU64>,
    last_cmd: Option<CommandId>,
    key_intercepted: bool,
}

impl TestDelegate {
    fn new() -> Self {
        Self {
            tick_count: Arc::new(AtomicU64::new(0)),
            last_cmd: None,
            key_intercepted: false,
        }
    }

    fn with_counter(count: Arc<AtomicU64>) -> Self {
        Self {
            tick_count: count,
            last_cmd: None,
            key_intercepted: false,
        }
    }
}

impl EditorViewDelegate for TestDelegate {
    fn on_tick(&mut self, _editor: &mut Editor, tick: u64) -> HandleResult {
        self.tick_count.store(tick, Ordering::Relaxed);
        HandleResult::Ignored
    }

    fn on_command(&mut self, id: CommandId, _data: &Option<Box<dyn Any + Send>>, _editor: &mut Editor) -> HandleResult {
        self.last_cmd = Some(id);
        HandleResult::Consumed
    }

    fn on_key_pre(&mut self, key: &KeyEvent, _editor: &mut Editor) -> Option<HandleResult> {
        if key.modifiers().ctrl() && key.code() == KeyCode::Char('n') {
            self.key_intercepted = true;
            return Some(HandleResult::Consumed);
        }
        None
    }
}

#[test]
fn delegate_on_tick_called() {
    let count = Arc::new(AtomicU64::new(0));
    let mut ev = EditorView::with_delegate(TestDelegate::with_counter(count.clone()));
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    ev.handle(&Event::Tick);
    ev.handle(&Event::Tick);
    assert_eq!(count.load(Ordering::Relaxed), 2);
}

#[test]
fn delegate_on_command_intercepts() {
    let mut ev = EditorView::with_delegate(TestDelegate::new());
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

#[test]
fn delegate_on_key_pre_intercepts() {
    let mut ev = EditorView::with_delegate(TestDelegate::new());
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    ev.handle(&Event::Key(KeyEvent::new(KeyCode::Char('n'), KeyMod::CTRL)));
    assert!(ev.delegate().key_intercepted);
}

#[test]
fn delegate_on_key_pre_passes_through() {
    let mut ev = EditorView::with_delegate(TestDelegate::new());
    ev.set_bounds(Rect::new(0, 0, 80, 24));
    ev.handle(&Event::Key(KeyEvent::new(KeyCode::Char('j'), KeyMod::NONE)));
    assert!(!ev.delegate().key_intercepted);
}
