//! Tests for ModalKey.

use txv_core::prelude::*;

use crate::modal_key::ModalKey;
use crate::InputLine;
use crate::KeyLabelView;

fn ctrl_w() -> KeyEvent {
    KeyEvent {
        code: KeyCode::Char('w'),
        modifiers: KeyMod {
            ctrl: true,
            alt: false,
            shift: false,
        },
    }
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyMod::default(),
    }
}

fn setup_prefix() -> ModalKey {
    ModalKey::new("C-w", "C-w: ")
        .trigger_key(ctrl_w())
        .cancel_on_miss()
        .add_child(Box::new(KeyLabelView::new(key(KeyCode::Char('s')), 100, "split")))
        .add_child(Box::new(KeyLabelView::new(key(KeyCode::Char('v')), 101, "vsplit")))
}

#[test]
fn dormant_shows_idle_label() {
    let mk = setup_prefix();
    assert_eq!(mk.bounds().w, 5); // "C-w" + 2 padding
}

#[test]
fn activates_on_trigger_key() {
    let mut mk = setup_prefix();
    mk.set_sink(EventSink::new());
    let result = mk.handle(&Event::Key(ctrl_w()));
    assert_eq!(result, HandleResult::Consumed);
    assert!(mk.bounds().w > 5); // expanded
}

#[test]
fn deactivates_on_child_command() {
    let mut mk = ModalKey::new("M-x", ":")
        .trigger_key(key(KeyCode::Char('x')))
        .add_child(Box::new(InputLine::new()));
    mk.set_sink(EventSink::new());
    mk.set_bounds(Rect::new(0, 0, 80, 1));

    // Activate
    mk.handle(&Event::Key(key(KeyCode::Char('x'))));
    assert!(mk.bounds().w > 0);

    // Type and press Enter → InputLine emits CM_OK
    mk.handle(&Event::Key(key(KeyCode::Char('h'))));
    mk.handle(&Event::Key(key(KeyCode::Enter)));

    // Should be deactivated now (back to dormant width)
    assert_eq!(mk.bounds().w, 5); // "M-x" + 2
}

#[test]
fn cancel_on_miss_deactivates() {
    let mut mk = setup_prefix();
    mk.set_sink(EventSink::new());

    mk.handle(&Event::Key(ctrl_w()));
    // Press unbound key
    mk.handle(&Event::Key(key(KeyCode::Char('z'))));
    // Should deactivate
    assert_eq!(mk.bounds().w, 5);
}

#[test]
fn input_line_tab_completes() {
    use crate::InputLine;

    struct TestCompleter;
    impl Completer for TestCompleter {
        fn complete(&self, input: &str, _cursor: usize) -> Vec<Completion> {
            if input == "he" {
                vec![Completion::new("help".into(), "help".into(), "cmd")]
            } else {
                vec![]
            }
        }
    }

    let input = InputLine::new()
        .with_command(100)
        .with_completer(Box::new(TestCompleter));
    let mut mk = ModalKey::new("M-x", ":")
        .trigger_key(key(KeyCode::Char('x')))
        .add_child(Box::new(input));
    mk.set_sink(EventSink::new());

    // Activate
    mk.handle(&Event::Key(key(KeyCode::Char('x'))));
    // Type "he"
    mk.handle(&Event::Key(key(KeyCode::Char('h'))));
    mk.handle(&Event::Key(key(KeyCode::Char('e'))));
    // Tab
    mk.handle(&Event::Key(key(KeyCode::Tab)));

    // Verify completion expanded the width (prompt ":" + "help" + padding)
    assert!(
        mk.bounds().w > 5,
        "expected expanded width after completion, got: {}",
        mk.bounds().w
    );
}
