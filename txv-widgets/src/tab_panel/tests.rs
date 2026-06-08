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
fn take_active_returns_view() -> Result<(), Box<dyn std::error::Error>> {
    let mut panel = TabPanel::new(TabBarMode::Static);
    panel.insert_tab("X", Box::new(Dummy::new()));
    panel.insert_tab("Y", Box::new(Dummy::new()));
    panel.set_active(0);

    let taken = panel.take_active();
    let (title, _) = taken.ok_or("take_active returned None")?;
    assert_eq!(title, "X");
    assert_eq!(panel.tab_count(), 1);
    Ok(())
}

#[test]
fn layout_gives_child_content_rect() -> Result<(), Box<dyn std::error::Error>> {
    let mut panel = TabPanel::new(TabBarMode::Static);
    panel.insert_tab("A", Box::new(Dummy::new()));
    panel.set_bounds(Rect::new(0, 0, 80, 24));

    let child = panel.active_child().ok_or("no active child")?;
    let cb = child.bounds();
    let (ox, oy) = panel.active_child_origin();
    assert_eq!(ox, 0);
    assert_eq!(oy, 1); // below tab bar
    assert_eq!(cb.w(), 80);
    assert_eq!(cb.h(), 23); // 24 - 1 for tab bar
    Ok(())
}

#[test]
fn draw_does_not_panic() {
    let mut panel = TabPanel::new(TabBarMode::Lru);
    panel.insert_tab("Shell", Box::new(Dummy::new()));
    panel.insert_tab("Build", Box::new(Dummy::new()));
    panel.set_bounds(Rect::new(0, 0, 60, 20));
    panel.render();
}

#[test]
fn m_digit_switches_tab() {
    let mut panel = TabPanel::new(TabBarMode::Static);
    panel.insert_tab("A", Box::new(Dummy::new()));
    panel.insert_tab("B", Box::new(Dummy::new()));
    panel.insert_tab("C", Box::new(Dummy::new()));
    panel.set_bounds(Rect::new(0, 0, 60, 20));

    let key = Event::Key(KeyEvent::new(KeyCode::Char('2'), KeyMod::ALT));
    panel.handle(&key);
    assert_eq!(panel.active_index(), 1);
}

#[test]
fn row_0_non_tab_cells_are_transparent() {
    let mut panel = TabPanel::new(TabBarMode::Single);
    panel.insert_tab("Hi", Box::new(Dummy::new()));
    panel.set_bounds(Rect::new(0, 0, 40, 10));
    panel.render();

    let buf = panel.buffer();
    // Check a position well past the tab content (should be transparent)
    let cell = buf.cell(30, 0);
    assert_eq!(
        cell.style().fg(),
        Color::Transparent,
        "non-tab cell at x=30 should have transparent fg, got {:?} ch={:?}",
        cell.style().fg(),
        cell.ch()
    );
    assert_eq!(
        cell.style().bg(),
        Color::Transparent,
        "non-tab cell at x=30 should have transparent bg"
    );
}

#[test]
fn needs_redraw_propagates_from_active_child() -> Result<(), Box<dyn std::error::Error>> {
    let mut panel = TabPanel::new(TabBarMode::Static);
    panel.insert_tab("A", Box::new(Dummy::new()));
    panel.set_bounds(Rect::new(0, 0, 80, 24));

    // After set_bounds, panel is dirty
    assert!(panel.needs_redraw());
    panel.mark_redrawn();
    assert!(!panel.needs_redraw());

    // Mark child dirty — panel should report needs_redraw
    panel
        .active_child_mut()
        .ok_or("no active child")?
        .set_bounds(Rect::new(0, 0, 79, 22));
    assert!(panel.needs_redraw());
    Ok(())
}

#[test]
fn tab_next_cycles() {
    let mut panel = TabPanel::new(TabBarMode::Static);
    panel.set_bounds(Rect::new(0, 0, 80, 24));
    panel.insert_tab("A", Box::new(Dummy::new()));
    panel.insert_tab("B", Box::new(Dummy::new()));
    panel.insert_tab("C", Box::new(Dummy::new()));
    panel.set_active(0);
    assert_eq!(panel.active_title(), Some("A"));
    panel.tab_next();
    assert_eq!(panel.active_title(), Some("B"));
    panel.tab_next();
    assert_eq!(panel.active_title(), Some("C"));
}

#[test]
fn close_tab_by_title() {
    let mut panel = TabPanel::new(TabBarMode::Static);
    panel.set_bounds(Rect::new(0, 0, 80, 24));
    panel.insert_tab("X", Box::new(Dummy::new()));
    panel.insert_tab("Y", Box::new(Dummy::new()));
    assert!(panel.close_tab_by_title("X"));
    assert_eq!(panel.tab_count(), 1);
    assert_eq!(panel.active_title(), Some("Y"));
}

#[test]
fn focus_tab_by_title() {
    let mut panel = TabPanel::new(TabBarMode::Static);
    panel.set_bounds(Rect::new(0, 0, 80, 24));
    panel.insert_tab("A", Box::new(Dummy::new()));
    panel.insert_tab("B", Box::new(Dummy::new()));
    assert!(panel.focus_tab_by_title("A"));
    assert_eq!(panel.active_title(), Some("A"));
    assert!(!panel.focus_tab_by_title("Z"));
}
