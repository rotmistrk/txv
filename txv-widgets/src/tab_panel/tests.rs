//! Tests for TabPanel.

use txv_core::prelude::*;

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

#[test]
fn insert_and_active() {
    let mut panel = TabPanel::new(TabBarMode::Static);
    panel.insert_tab("A", Box::new(Dummy::new()));
    panel.insert_tab("B", Box::new(Dummy::new()));
    assert_eq!(panel.tab_count(), 2);
    assert_eq!(panel.active_index(), 1); // last inserted is active

    panel.set_active(0);
    assert_eq!(panel.active_index(), 0);
}

#[test]
fn remove_tab_adjusts_active() {
    let mut panel = TabPanel::new(TabBarMode::Lru);
    panel.insert_tab("A", Box::new(Dummy::new()));
    panel.insert_tab("B", Box::new(Dummy::new()));
    panel.insert_tab("C", Box::new(Dummy::new()));
    panel.set_active(2);

    panel.remove_tab(0);
    assert_eq!(panel.tab_count(), 2);
    // Active was 2, now shifted to 1
    assert_eq!(panel.active_index(), 1);
}

#[test]
fn take_active_returns_view() {
    let mut panel = TabPanel::new(TabBarMode::Static);
    panel.insert_tab("X", Box::new(Dummy::new()));
    panel.insert_tab("Y", Box::new(Dummy::new()));
    panel.set_active(0);

    let taken = panel.take_active();
    assert!(taken.is_some());
    let (title, _) = taken.unwrap();
    assert_eq!(title, "X");
    assert_eq!(panel.tab_count(), 1);
}

#[test]
fn layout_gives_child_content_rect() {
    let mut panel = TabPanel::new(TabBarMode::Static);
    panel.insert_tab("A", Box::new(Dummy::new()));
    panel.set_bounds(Rect::new(0, 0, 80, 24));

    let child = panel.active_child().unwrap();
    let cb = child.bounds();
    assert_eq!(cb.x, 0);
    assert_eq!(cb.y, 1); // below tab bar
    assert_eq!(cb.w, 80);
    assert_eq!(cb.h, 23); // 24 - 1 for tab bar
}

#[test]
fn draw_does_not_panic() {
    let mut panel = TabPanel::new(TabBarMode::Lru);
    panel.insert_tab("Shell", Box::new(Dummy::new()));
    panel.insert_tab("Build", Box::new(Dummy::new()));
    panel.set_bounds(Rect::new(0, 0, 60, 20));
    panel.draw();
}

#[test]
fn m_digit_switches_tab() {
    let mut panel = TabPanel::new(TabBarMode::Static);
    panel.insert_tab("A", Box::new(Dummy::new()));
    panel.insert_tab("B", Box::new(Dummy::new()));
    panel.insert_tab("C", Box::new(Dummy::new()));
    panel.set_bounds(Rect::new(0, 0, 60, 20));

    let key = Event::Key(KeyEvent {
        code: KeyCode::Char('2'),
        modifiers: KeyMod {
            alt: true,
            ctrl: false,
            shift: false,
        },
    });
    panel.handle(&key);
    assert_eq!(panel.active_index(), 1);
}
