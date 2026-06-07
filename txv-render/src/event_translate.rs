//! Crossterm event translation — converts crossterm events to txv-core events.

use crossterm::event as ct_event;
use crossterm::event::{
    KeyCode as CtKeyCode, KeyEventKind, KeyModifiers, MouseButton as CtMouseButton, MouseEvent as CtMouseEvent,
    MouseEventKind,
};
use txv_core::event::{Event, KeyCode, KeyEvent, KeyMod, MouseAction, MouseButton, MouseEvent};

pub(crate) fn translate_key(key: ct_event::KeyEvent) -> Option<Event> {
    if key.kind == KeyEventKind::Release {
        return None;
    }

    let mut modifiers = KeyMod::NONE;
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        modifiers = modifiers.with_ctrl();
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        modifiers = modifiers.with_alt();
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        modifiers = modifiers.with_shift();
    }

    let code = match key.code {
        CtKeyCode::Char(c) => KeyCode::Char(c),
        CtKeyCode::F(n) => KeyCode::F(n),
        CtKeyCode::Enter => KeyCode::Enter,
        CtKeyCode::Esc => KeyCode::Esc,
        CtKeyCode::Tab => KeyCode::Tab,
        CtKeyCode::BackTab => KeyCode::BackTab,
        CtKeyCode::Backspace => KeyCode::Backspace,
        CtKeyCode::Delete => KeyCode::Delete,
        CtKeyCode::Left => KeyCode::Left,
        CtKeyCode::Right => KeyCode::Right,
        CtKeyCode::Up => KeyCode::Up,
        CtKeyCode::Down => KeyCode::Down,
        CtKeyCode::Home => KeyCode::Home,
        CtKeyCode::End => KeyCode::End,
        CtKeyCode::PageUp => KeyCode::PageUp,
        CtKeyCode::PageDown => KeyCode::PageDown,
        CtKeyCode::Insert => KeyCode::Insert,
        _ => return None,
    };

    Some(Event::Key(KeyEvent::new(code, modifiers)))
}

pub(crate) fn translate_mouse(m: CtMouseEvent) -> Option<Event> {
    let mut modifiers = KeyMod::NONE;
    if m.modifiers.contains(KeyModifiers::CONTROL) {
        modifiers = modifiers.with_ctrl();
    }
    if m.modifiers.contains(KeyModifiers::ALT) {
        modifiers = modifiers.with_alt();
    }
    if m.modifiers.contains(KeyModifiers::SHIFT) {
        modifiers = modifiers.with_shift();
    }

    let action = match m.kind {
        MouseEventKind::Down(CtMouseButton::Left) => MouseAction::Press(MouseButton::Left),
        MouseEventKind::Down(CtMouseButton::Right) => MouseAction::Press(MouseButton::Right),
        MouseEventKind::Down(CtMouseButton::Middle) => MouseAction::Press(MouseButton::Middle),
        MouseEventKind::Up(CtMouseButton::Left) => MouseAction::Release(MouseButton::Left),
        MouseEventKind::Up(CtMouseButton::Right) => MouseAction::Release(MouseButton::Right),
        MouseEventKind::Up(CtMouseButton::Middle) => MouseAction::Release(MouseButton::Middle),
        MouseEventKind::Moved | MouseEventKind::Drag(_) => MouseAction::Move,
        MouseEventKind::ScrollUp => MouseAction::ScrollUp,
        MouseEventKind::ScrollDown => MouseAction::ScrollDown,
        _ => return None,
    };

    Some(Event::Mouse(MouseEvent::new(m.column, m.row, action, modifiers)))
}
