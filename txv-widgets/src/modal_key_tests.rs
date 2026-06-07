//! Tests for ModalKey.

use txv_core::palette::palette;
use txv_core::prelude::*;

use crate::modal_key::ModalKey;
use crate::InputLine;
use crate::KeyLabelView;

fn ctrl_w() -> KeyEvent {
    KeyEvent::new(KeyCode::Char('w'), KeyMod::CTRL)
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyMod::default())
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
    assert_eq!(mk.bounds().w(), 5); // "C-w" + 2 padding
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
    let modal_bg = palette().style(StyleId::StatusBarModal).bg();
    assert_eq!(buf.cell(2, 0).style().bg(), modal_bg, "should be in active/modal state");
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
    assert!(mk.bounds().w() > 0);

    // Type and press Enter → InputLine emits CM_OK
    mk.handle(&Event::Key(key(KeyCode::Char('h'))));
    mk.handle(&Event::Key(key(KeyCode::Enter)));

    // Should be deactivated now (back to dormant width)
    assert_eq!(mk.bounds().w(), 5); // "M-x" + 2
}

#[test]
fn cancel_on_miss_deactivates() {
    let mut mk = setup_prefix();
    mk.set_sink(EventSink::new());

    mk.handle(&Event::Key(ctrl_w()));
    // Press unbound key
    mk.handle(&Event::Key(key(KeyCode::Char('z'))));
    // Should deactivate
    assert_eq!(mk.bounds().w(), 5);
}

#[test]
fn input_line_tab_completes() {
    use crate::InputLine;

    let mk = setup_completion_modal();
    let text = activate_type_tab_and_read(mk);
    assert!(text.contains("help"), "expected 'help' in buffer, got: {}", text.trim());
}

fn setup_completion_modal() -> ModalKey {
    use crate::modal_key_test_helpers::TestCompleter;
    use crate::InputLine;

    let input = InputLine::new()
        .with_command(100)
        .with_completer(Box::new(TestCompleter));
    ModalKey::new("M-x", ":")
        .trigger_key(key(KeyCode::Char('x')))
        .add_child(Box::new(input))
}

fn activate_type_tab_and_read(mut mk: ModalKey) -> String {
    mk.set_sink(EventSink::new());
    mk.set_bounds(Rect::new(0, 0, 80, 1));
    mk.handle(&Event::Key(key(KeyCode::Char('x'))));
    mk.handle(&Event::Key(key(KeyCode::Char('h'))));
    mk.handle(&Event::Key(key(KeyCode::Char('e'))));
    mk.handle(&Event::Key(key(KeyCode::Tab)));
    mk.draw();
    let buf = mk.buffer();
    let mut text = String::new();
    for x in 0..buf.width() {
        text.push(buf.cell(x, 0).ch());
    }
    text
}

#[test]
fn active_children_use_modal_background() {
    let mut mk = setup_prefix();
    mk.set_sink(EventSink::new());
    mk.set_bounds(Rect::new(0, 0, 80, 1));

    // Activate
    mk.handle(&Event::Key(ctrl_w()));
    mk.draw();

    let modal_bg = palette().style(StyleId::StatusBarModal).bg();
    let buf = mk.buffer();

    // Check cells inside the modal (after left cap, before right cap)
    // Prompt "C-w: " starts at x=1, children follow
    // Cell at x=2 should have modal bg (part of prompt)
    assert_eq!(
        buf.cell(2, 0).style().bg(),
        modal_bg,
        "prompt area must have modal background"
    );

    // Children area (after prompt) should also have modal bg
    let prompt_end = 1 + "C-w: ".len() as u16;
    assert_eq!(
        buf.cell(prompt_end + 1, 0).style().bg(),
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

    let modal_bg = palette().style(StyleId::StatusBarModal).bg();
    let buf = mk.buffer();

    // Check the first typed char (not cursor position)
    // Layout: [cap][:][ input content... ][cap]
    // cap=1, prompt ":"=1, then input starts
    // Input "hi" with cursor at pos 2 — check pos 0 of input ("h")
    let input_x = 2; // after cap + prompt
    assert_eq!(
        buf.cell(input_x, 0).style().bg(),
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

    let bar_bg = palette().style(StyleId::StatusBar).bg();
    let buf = mk.buffer();

    // Dormant: shows "C-w" with status bar bg
    assert_eq!(
        buf.cell(1, 0).style().bg(),
        bar_bg,
        "dormant label must have status bar background"
    );
}

// === Regression: deactivate zeros children bounds to prevent ghost rendering (0a20294) ===

#[test]
fn deactivate_zeros_children_bounds() {
    let mut mk = setup_prefix();
    mk.set_sink(EventSink::new());
    mk.set_bounds(Rect::new(0, 0, 80, 1));

    // Activate
    mk.handle(&Event::Key(ctrl_w()));

    // Children should have non-zero bounds when active
    let child_bounds = mk.group.child(0).map(|c| c.bounds());
    assert!(
        child_bounds.is_some_and(|r| r.w() > 0),
        "active children should have non-zero width"
    );

    // Deactivate via cancel_on_miss
    mk.handle(&Event::Key(key(KeyCode::Char('z'))));

    // After deactivation, children bounds should be zero
    let child_bounds = mk.group.child(0).map(|c| c.bounds());
    assert!(
        child_bounds.is_some_and(|r| r.w() == 0 && r.h() == 0),
        "deactivated children must have zero bounds: {:?}",
        child_bounds
    );
}
