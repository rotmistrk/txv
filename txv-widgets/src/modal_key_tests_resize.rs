//! ModalKey: an active prompt's input follows the width the StatusBar gives it.
//!
//! The StatusBar sizes the ModalKey after it activates (stretch distribution).
//! The input child must track those bounds, not the width at activation time.

use txv_core::prelude::*;

use crate::modal_key::ModalKey;
use crate::InputLine;

const PROMPT: &str = ":";
/// Prompt plus left and right caps.
const CHROME: u16 = PROMPT.len() as u16 + 2;

fn alt_x() -> KeyEvent {
    KeyEvent::new(KeyCode::Char('x'), KeyMod::ALT)
}

fn command_line() -> ModalKey {
    let mut mk = ModalKey::new("M-x", PROMPT)
        .trigger_key(alt_x())
        .add_child(Box::new(InputLine::new().with_constrained()));
    mk.set_sink(EventSink::new());
    mk
}

fn input_width(mk: &ModalKey) -> u16 {
    mk.group.child(0).map_or(0, |c| c.bounds().w())
}

#[test]
fn input_grows_when_bar_widens_active_prompt() {
    let mut mk = command_line();
    mk.set_bounds(Rect::new(0, 0, 5, 1));
    mk.handle(&Event::Key(alt_x()));
    mk.set_bounds(Rect::new(10, 0, 60, 1));
    assert_eq!(input_width(&mk), 60 - CHROME);
}

#[test]
fn input_shrinks_when_bar_narrows_active_prompt() {
    let mut mk = command_line();
    mk.set_bounds(Rect::new(0, 0, 80, 1));
    mk.handle(&Event::Key(alt_x()));
    mk.set_bounds(Rect::new(0, 0, 30, 1));
    assert_eq!(input_width(&mk), 30 - CHROME);
}

#[test]
fn dormant_resize_keeps_children_hidden() {
    let mut mk = command_line();
    mk.set_bounds(Rect::new(0, 0, 60, 1));
    assert_eq!(input_width(&mk), 0, "dormant prompt must not lay out its input");
}

#[test]
fn reopened_prompt_shows_full_history_entry() {
    let mut mk = command_line();
    mk.set_bounds(Rect::new(0, 0, 60, 1));
    mk.handle(&Event::Key(alt_x()));
    for ch in "test".chars() {
        mk.handle(&Event::Key(KeyEvent::new(KeyCode::Char(ch), KeyMod::NONE)));
    }
    mk.handle(&Event::Key(KeyEvent::new(KeyCode::Enter, KeyMod::NONE)));
    // Dormant again: the bar shrinks it to its idle label, then re-stretches after reopen.
    mk.set_bounds(Rect::new(0, 0, 5, 1));
    mk.handle(&Event::Key(alt_x()));
    mk.set_bounds(Rect::new(0, 0, 40, 1));
    assert_eq!(input_width(&mk), 40 - CHROME);
}
