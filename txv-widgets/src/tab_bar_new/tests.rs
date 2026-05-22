//! Tests for the new TabBar widget.

use txv_core::prelude::*;

use crate::tab_bar::{TabBar, TabBarMode};

#[test]
fn static_mode_all_numbered() {
    let mut bar = TabBar::new(TabBarMode::Static);
    bar.add_tab("Files");
    bar.add_tab("Git");
    bar.add_tab("Tools");
    bar.set_active(1);

    let order = bar.display_order();
    assert_eq!(order, vec![0, 1, 2]);

    // All tabs get numbers in static mode
    assert_eq!(bar.number_label(0, 0), Some('₁'));
    assert_eq!(bar.number_label(1, 1), Some('₂'));
    assert_eq!(bar.number_label(2, 2), Some('₃'));
}

#[test]
fn lru_mode_active_first_no_number() {
    let mut bar = TabBar::new(TabBarMode::Lru);
    bar.add_tab("a.rs");
    bar.add_tab("b.rs");
    bar.add_tab("c.rs");
    bar.set_active(1); // b.rs is active

    let order = bar.display_order();
    assert_eq!(order[0], 1, "active should be first");

    // Active has no number
    assert_eq!(bar.number_label(0, 1), None);
    // Others get numbers
    assert!(bar.number_label(1, 0).is_some());
}

#[test]
fn single_mode_only_active() {
    let mut bar = TabBar::new(TabBarMode::Single);
    bar.add_tab("Shell");
    bar.add_tab("Build");
    bar.add_tab("Output");
    bar.set_active(0);

    let order = bar.display_order();
    assert_eq!(order, vec![0], "single mode shows only active");
}

#[test]
fn draw_does_not_panic() {
    let mut bar = TabBar::new(TabBarMode::Static);
    bar.add_tab("Files");
    bar.add_tab("Git");
    bar.set_active(0);
    bar.set_bounds(Rect::new(0, 0, 40, 1));
    bar.draw();
}

#[test]
fn draw_single_does_not_panic() {
    let mut bar = TabBar::new(TabBarMode::Single);
    bar.add_tab("Shell");
    bar.add_tab("Build");
    bar.set_active(0);
    bar.set_bounds(Rect::new(0, 0, 40, 1));
    bar.draw();
}

#[test]
fn dropdown_entries_lru_active_no_number() {
    let mut bar = TabBar::new(TabBarMode::Lru);
    bar.add_tab("a.rs");
    bar.add_tab("b.rs");
    bar.add_tab("c.rs");
    bar.set_active(1);

    let entries = bar.dropdown_entries();
    // Active entry has no "N:" prefix
    assert!(!entries[0].1.contains(':'), "active should have no number prefix");
    // Others have "N:" prefix
    assert!(entries[1].1.contains(':'));
}

#[test]
fn dropdown_filter_narrows_results() {
    let mut bar = TabBar::new(TabBarMode::Static);
    bar.add_tab("main.rs");
    bar.add_tab("lib.rs");
    bar.add_tab("test.rs");
    bar.set_active(0);

    bar.open_dropdown();
    bar.dropdown_filter = "lib".to_string();
    let entries = bar.dropdown_entries();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, 1);
}

#[test]
fn remove_tab_adjusts_state() {
    let mut bar = TabBar::new(TabBarMode::Static);
    bar.add_tab("a");
    bar.add_tab("b");
    bar.add_tab("c");
    bar.set_active(2);
    bar.remove_tab(0);
    assert_eq!(bar.tab_count(), 2);
    assert_eq!(bar.active_index(), 1); // was 2, shifted down
}

#[test]
fn activate_by_number_static() {
    let mut bar = TabBar::new(TabBarMode::Static);
    bar.add_tab("a");
    bar.add_tab("b");
    bar.add_tab("c");
    bar.set_active(0);

    // M-2 should activate tab index 1
    let key = Event::Key(KeyEvent {
        code: KeyCode::Char('2'),
        modifiers: KeyMod {
            alt: true,
            ctrl: false,
            shift: false,
        },
    });
    bar.handle(&key);
    assert_eq!(bar.active_index(), 1);
}
