//! TabPanel identity tests: after any remove, the active title, the focused
//! child, and the only visible child must be the SAME tab. Index arithmetic
//! that keeps them internally consistent but on the wrong tab is a failure.

use std::any::Any;

use txv_core::prelude::*;

use crate::tab_bar::TabBarMode;
use crate::tab_panel::TabPanel;

/// A view that knows which tab it belongs to.
struct Labeled {
    state: ViewState,
    label: String,
}

impl Labeled {
    fn boxed(label: &str) -> Box<dyn View> {
        Box::new(Self {
            state: ViewState::default(),
            label: label.to_string(),
        })
    }
}

impl View for Labeled {
    delegate_view_state!(state);
    fn draw(&mut self) {}
    fn handle(&mut self, _: &Event) -> HandleResult {
        HandleResult::Ignored
    }
    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

const FILES: &[&str] = &[
    "main.rs",
    "lib.rs",
    "open.rs",
    "close.rs",
    "a/mod.rs",
    "b/mod.rs",
    "README.md",
    "guide.md",
    "users.csv",
    "settings.json",
    "Cargo.toml",
    "build.rs",
];

fn panel_with_all_files(mode: TabBarMode) -> TabPanel {
    let mut panel = TabPanel::new(mode);
    panel.set_bounds(Rect::new(0, 0, 120, 40));
    for f in FILES {
        panel.insert_tab(*f, Labeled::boxed(f));
    }
    panel
}

fn label_at(panel: &TabPanel, group_index: usize) -> Option<String> {
    panel
        .group
        .child(group_index)
        .and_then(|v| v.as_any())
        .and_then(|a| a.downcast_ref::<Labeled>())
        .map(|l| l.label.clone())
}

/// Every title must sit on its own view, and the active title, the focused
/// child, and the single visible child must all be the same tab.
fn assert_consistent(panel: &TabPanel, context: &str) {
    for i in 0..panel.tab_count() {
        let title = panel.tab_title(i).map(str::to_string);
        assert_eq!(title, label_at(panel, i + 1), "{context}: title/view mismatch at {i}");
    }
    let active = panel.active_index();
    let active_title = panel.active_title().map(str::to_string);
    assert_eq!(
        label_at(panel, panel.group.focused_index()),
        active_title,
        "{context}: focused child is not the active tab"
    );
    let visible: Vec<usize> = (1..panel.group.child_count())
        .filter(|&gi| panel.group.is_child_visible(gi))
        .collect();
    assert_eq!(
        visible,
        vec![active + 1],
        "{context}: exactly the active tab must be visible"
    );
}

#[test]
fn removing_tab_left_of_active_keeps_the_same_active_file() {
    for active in 1..FILES.len() {
        for removed in 0..active {
            let mut panel = panel_with_all_files(TabBarMode::Static);
            panel.set_active(active);
            let expected = FILES[active];

            panel.remove_tab(removed);

            let ctx = format!("active {active}, removed {removed}");
            assert_eq!(panel.active_title(), Some(expected), "{ctx}");
            assert_consistent(&panel, &ctx);
        }
    }
}

#[test]
fn removing_tab_right_of_active_keeps_the_same_active_file() {
    for active in 0..FILES.len() - 1 {
        for removed in active + 1..FILES.len() {
            let mut panel = panel_with_all_files(TabBarMode::Static);
            panel.set_active(active);

            panel.remove_tab(removed);

            let ctx = format!("active {active}, removed {removed}");
            assert_eq!(panel.active_title(), Some(FILES[active]), "{ctx}");
            assert_consistent(&panel, &ctx);
        }
    }
}

#[test]
fn closing_the_active_tab_activates_a_neighbor_consistently() {
    for active in 0..FILES.len() {
        let mut panel = panel_with_all_files(TabBarMode::Static);
        panel.set_active(active);

        panel.close_active();

        let ctx = format!("closed active {active}");
        let expected = if active + 1 < FILES.len() {
            FILES[active + 1]
        } else {
            FILES[active - 1]
        };
        assert_eq!(panel.active_title(), Some(expected), "{ctx}");
        assert_consistent(&panel, &ctx);
    }
}

#[test]
fn closing_tabs_one_by_one_in_lru_mode_stays_consistent() {
    let mut panel = panel_with_all_files(TabBarMode::Lru);
    // Realistic history: jump around before closing.
    for i in [3, 9, 0, 7, 11, 5, 2] {
        panel.set_active(i);
    }
    let mut step = 0;
    while panel.tab_count() > 1 {
        step += 1;
        let victim = (step * 5) % panel.tab_count();
        if step % 2 == 0 {
            panel.set_active(victim);
            panel.close_active();
        } else {
            panel.remove_tab(victim);
        }
        assert_consistent(&panel, &format!("step {step}"));
    }
}

#[test]
fn take_active_leaves_panel_consistent() {
    let mut panel = panel_with_all_files(TabBarMode::Static);
    panel.set_active(4);
    let taken = panel.take_active().map(|(title, _)| title);
    assert_eq!(taken.as_deref(), Some(FILES[4]));
    assert_eq!(panel.active_title(), Some(FILES[5]));
    assert_consistent(&panel, "after take_active");
}

#[test]
fn insert_while_dropdown_open_keeps_titles_on_their_views() {
    let mut panel = panel_with_all_files(TabBarMode::Static);
    panel.open_dropdown();
    panel.insert_tab("late.rs", Labeled::boxed("late.rs"));
    for i in 0..panel.tab_count() {
        assert_eq!(
            panel.tab_title(i).map(str::to_string),
            label_at(&panel, i + 1),
            "tab {i}"
        );
    }
}
