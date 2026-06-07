//! Events flowing through the view tree.

mod key_event;
mod key_mod;
mod mouse_event;

use std::any::Any;

pub use key_event::KeyEvent;
pub use key_mod::KeyMod;
pub use mouse_event::MouseEvent;

/// Command identifier type.
pub type CommandId = u16;

/// Key codes (terminal-independent).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyCode {
    Char(char),
    F(u8),
    Enter,
    Esc,
    Tab,
    BackTab,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
}

/// Mouse button.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Mouse action.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MouseAction {
    Press(MouseButton),
    Release(MouseButton),
    Move,
    ScrollUp,
    ScrollDown,
}

/// An event flowing through the view tree.
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Paste(String),
    Resize(u16, u16),
    /// Command event. When `broadcast` is true, dispatched to ALL children (not just focused).
    Command {
        id: CommandId,
        data: Option<Box<dyn Any + Send>>,
        broadcast: bool,
    },
    Tick,
}

impl core::fmt::Debug for Event {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Key(k) => f.debug_tuple("Key").field(k).finish(),
            Self::Mouse(m) => f.debug_tuple("Mouse").field(m).finish(),
            Self::Paste(s) => f.debug_tuple("Paste").field(&s.len()).finish(),
            Self::Resize(w, h) => f.debug_tuple("Resize").field(w).field(h).finish(),
            Self::Command { id, broadcast, .. } => f
                .debug_struct("Command")
                .field("id", id)
                .field("broadcast", broadcast)
                .finish(),
            Self::Tick => write!(f, "Tick"),
        }
    }
}
