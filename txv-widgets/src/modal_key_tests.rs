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
    mk.set_bounds(Rect::new(0, 0, 80, 1));
    let result = mk.handle(&Event::Key(ctrl_w()));
    assert_eq!(result, HandleResult::Consumed);
    // After activation, draw shows children (not just idle label)
    mk.draw();
    let buf = mk.buffer();
    // Should show "C-w: " prompt (modal style)
    let modal_bg = txv_core::palette::palette().style(StyleId::StatusBarModal).bg;
    assert_eq!(buf.cell(2, 0).style.bg, modal_bg, "should be in active/modal state");
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
        fn complete(
            &self,
            input: &str,
            _cursor: usize,
            visitor: &mut CompletionVisitor<'_>,
        ) -> Result<(), Box<dyn std::error::Error>> {
            if input == "he" {
                struct C;
                impl Completion for C {
                    fn text(&self) -> &str {
                        "help"
                    }
                    fn display(&self) -> &str {
                        "help"
                    }
                    fn kind(&self) -> &str {
                        "cmd"
                    }
                }
                visitor(&C)?;
            }
            Ok(())
        }
    }

    let input = InputLine::new()
        .with_command(100)
        .with_completer(Box::new(TestCompleter));
    let mut mk = ModalKey::new("M-x", ":")
        .trigger_key(key(KeyCode::Char('x')))
        .add_child(Box::new(input));
    mk.set_sink(EventSink::new());
    mk.set_bounds(Rect::new(0, 0, 80, 1));

    // Activate
    mk.handle(&Event::Key(key(KeyCode::Char('x'))));
    // Type "he"
    mk.handle(&Event::Key(key(KeyCode::Char('h'))));
    mk.handle(&Event::Key(key(KeyCode::Char('e'))));
    // Tab
    mk.handle(&Event::Key(key(KeyCode::Tab)));

    // Verify completion: draw and check that "help" appears in the buffer
    mk.draw();
    let buf = mk.buffer();
    let mut text = String::new();
    for x in 0..buf.width() {
        text.push(buf.cell(x, 0).ch);
    }
    assert!(text.contains("help"), "expected 'help' in buffer, got: {}", text.trim());
}

#[test]
fn active_children_use_modal_background() {
    let mut mk = setup_prefix();
    mk.set_sink(EventSink::new());
    mk.set_bounds(Rect::new(0, 0, 80, 1));

    // Activate
    mk.handle(&Event::Key(ctrl_w()));
    mk.draw();

    let modal_bg = txv_core::palette::palette().style(StyleId::StatusBarModal).bg;
    let buf = mk.buffer();

    // Check cells inside the modal (after left cap, before right cap)
    // Prompt "C-w: " starts at x=1, children follow
    // Cell at x=2 should have modal bg (part of prompt)
    assert_eq!(
        buf.cell(2, 0).style.bg,
        modal_bg,
        "prompt area must have modal background"
    );

    // Children area (after prompt) should also have modal bg
    let prompt_end = 1 + "C-w: ".len() as u16;
    assert_eq!(
        buf.cell(prompt_end + 1, 0).style.bg,
        modal_bg,
        "child area must have modal background"
    );
}

#[test]
fn active_input_line_uses_modal_background() {
    let input = InputLine::new().with_command(100);
    let mut mk = ModalKey::new("M-x", ":")
        .trigger_key(key(KeyCode::Char('x')))
        .add_child(Box::new(input));
    mk.set_sink(EventSink::new());
    mk.set_bounds(Rect::new(0, 0, 80, 1));

    // Activate
    mk.handle(&Event::Key(key(KeyCode::Char('x'))));
    // Type something so input has content beyond cursor
    mk.handle(&Event::Key(key(KeyCode::Char('h'))));
    mk.handle(&Event::Key(key(KeyCode::Char('i'))));
    mk.draw();

    let modal_bg = txv_core::palette::palette().style(StyleId::StatusBarModal).bg;
    let buf = mk.buffer();

    // Check the first typed char (not cursor position)
    // Layout: [cap][:][ input content... ][cap]
    // cap=1, prompt ":"=1, then input starts
    // Input "hi" with cursor at pos 2 — check pos 0 of input ("h")
    let input_x = 2; // after cap + prompt
    assert_eq!(
        buf.cell(input_x, 0).style.bg,
        modal_bg,
        "input line text must have modal background"
    );
}

#[test]
fn dormant_children_use_status_bar_background() {
    let mut mk = setup_prefix();
    mk.set_sink(EventSink::new());
    mk.set_bounds(Rect::new(0, 0, 80, 1));

    // Activate then deactivate
    mk.handle(&Event::Key(ctrl_w()));
    mk.handle(&Event::Key(key(KeyCode::Char('z')))); // cancel_on_miss
    mk.draw();

    let bar_bg = txv_core::palette::palette().style(StyleId::StatusBar).bg;
    let buf = mk.buffer();

    // Dormant: shows "C-w" with status bar bg
    assert_eq!(
        buf.cell(1, 0).style.bg,
        bar_bg,
        "dormant label must have status bar background"
    );
}
