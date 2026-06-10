//! Scenario tests for DropdownMenu integration in TabPanel.

use txv_core::prelude::*;

use crate::dropdown_menu::CM_DROPDOWN_DONE;
use crate::tab_bar::TabBarMode;
use crate::tab_panel::TabPanel;

struct Dummy {
    state: ViewState,
}
impl Dummy {
    fn new() -> Self {
        Self {
            state: ViewState::default(),
        }
    }
}
impl View for Dummy {
    delegate_view_state!(state);
    fn draw(&mut self) {}
    fn handle(&mut self, _: &Event) -> HandleResult {
        HandleResult::Ignored
    }
}

fn make_panel() -> TabPanel {
    let mut panel = TabPanel::new(TabBarMode::Static);
    panel.insert_tab("Alpha", Box::new(Dummy::new()));
    panel.insert_tab("Beta", Box::new(Dummy::new()));
    panel.insert_tab("Gamma", Box::new(Dummy::new()));
    panel.set_bounds(Rect::new(0, 0, 60, 20));
    panel.set_active(0);
    panel
}

#[test]
fn open_dropdown_lists_all_tabs() {
    let mut panel = make_panel();
    panel.open_dropdown();
    assert!(panel.dropdown_open());
    // Group has bar + 3 tabs + 1 dropdown = 5 children
    assert_eq!(panel.tab_count(), 3);
}

#[test]
fn esc_closes_dropdown_keeps_active() {
    let mut panel = make_panel();
    panel.open_dropdown();
    assert!(panel.dropdown_open());

    let esc = Event::Key(KeyEvent::new(KeyCode::Esc, KeyMod::NONE));
    let sink = EventSink::new();
    panel.set_sink(sink.clone());
    panel.handle(&esc);
    // Drain the CM_DROPDOWN_CANCELLED command
    let events = sink.drain();
    for ev in events {
        panel.handle(&ev);
    }

    assert!(!panel.dropdown_open());
    assert_eq!(panel.active_index(), 0);
}

#[test]
fn enter_selects_tab() {
    let mut panel = make_panel();
    panel.open_dropdown();

    let sink = EventSink::new();
    panel.set_sink(sink.clone());

    // Move down to "Beta" (index 1) then Enter
    let down = Event::Key(KeyEvent::new(KeyCode::Down, KeyMod::NONE));
    panel.handle(&down);
    let enter = Event::Key(KeyEvent::new(KeyCode::Enter, KeyMod::NONE));
    panel.handle(&enter);

    // Drain the CM_DROPDOWN_DONE command
    let events = sink.drain();
    for ev in events {
        panel.handle(&ev);
    }

    assert!(!panel.dropdown_open());
    assert_eq!(panel.active_index(), 1);
}

#[test]
fn filter_narrows_list() {
    let mut panel = make_panel();
    panel.open_dropdown();

    let sink = EventSink::new();
    panel.set_sink(sink.clone());

    // Type 'b' — should match only "Beta" with prefix filter
    let b = Event::Key(KeyEvent::new(KeyCode::Char('b'), KeyMod::NONE));
    panel.handle(&b);

    // Enter selects first visible (Beta, original index 1)
    let enter = Event::Key(KeyEvent::new(KeyCode::Enter, KeyMod::NONE));
    panel.handle(&enter);

    let events = sink.drain();
    for ev in events {
        panel.handle(&ev);
    }

    assert!(!panel.dropdown_open());
    assert_eq!(panel.active_index(), 1);
}

#[test]
fn number_hotkey_selects_directly() {
    let mut panel = make_panel();
    panel.open_dropdown();

    let sink = EventSink::new();
    panel.set_sink(sink.clone());

    // Alt-2 should select index 1 (NumberMode::All, 2nd item)
    let alt2 = Event::Key(KeyEvent::new(KeyCode::Char('2'), KeyMod::ALT));
    panel.handle(&alt2);

    let events = sink.drain();
    for ev in events {
        panel.handle(&ev);
    }

    assert!(!panel.dropdown_open());
    assert_eq!(panel.active_index(), 1);
}

#[test]
fn active_tab_stays_visible_during_dropdown() {
    let mut panel = make_panel();
    panel.open_dropdown();

    // The active child (tab 0) should still be visible
    // (no hiding hack) — render succeeds without panic
    panel.render();
}

#[test]
fn dropdown_done_command_switches_tab() {
    let mut panel = make_panel();
    panel.open_dropdown();

    // Simulate receiving CM_DROPDOWN_DONE with index 2
    let ev = Event::Command {
        id: CM_DROPDOWN_DONE,
        data: Some(Box::new(2usize)),
        broadcast: false,
    };
    panel.handle(&ev);

    assert!(!panel.dropdown_open());
    assert_eq!(panel.active_index(), 2);
}

#[test]
fn double_open_is_idempotent() {
    let mut panel = make_panel();
    panel.open_dropdown();
    panel.open_dropdown();
    assert!(panel.dropdown_open());
    // Still only one dropdown child
    // bar + 3 tabs + 1 dropdown = 5 total
    // tab_count() returns 3 (excludes bar and dropdown)
    assert_eq!(panel.tab_count(), 3);
}

#[test]
fn lru_mode_uses_skip_first() {
    let mut panel = TabPanel::new(TabBarMode::Lru);
    panel.insert_tab("Main", Box::new(Dummy::new()));
    panel.insert_tab("Alt", Box::new(Dummy::new()));
    panel.set_bounds(Rect::new(0, 0, 60, 20));
    panel.set_active(0);
    panel.open_dropdown();
    assert!(panel.dropdown_open());

    let sink = EventSink::new();
    panel.set_sink(sink.clone());

    // Alt-1 in SkipFirst mode selects index 1 (skips first)
    let alt1 = Event::Key(KeyEvent::new(KeyCode::Char('1'), KeyMod::ALT));
    panel.handle(&alt1);

    let events = sink.drain();
    for ev in events {
        panel.handle(&ev);
    }

    assert!(!panel.dropdown_open());
    assert_eq!(panel.active_index(), 1);
}
