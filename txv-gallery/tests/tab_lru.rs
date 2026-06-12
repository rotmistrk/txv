//! Scenario tests for TabPanel in LRU mode — Alt-digit switching and dropdown order sync.

use txv_core::prelude::*;

/// Navigate to TabLRU demo (index 12), focus demo panel.
fn nav_to_demo(app: &mut dyn View, backend: &mut MockBackend) {
    for _ in 0..12 {
        backend.inject_key(KeyCode::Down, KeyMod::NONE);
        run_cycles(app, backend, 1);
    }
    backend.inject_key(KeyCode::Right, KeyMod::CTRL.with_shift());
    run_cycles(app, backend, 1);
}

/// Open dropdown and collect the displayed tab labels in order.
fn dropdown_labels(app: &mut dyn View, backend: &mut MockBackend) -> Vec<String> {
    backend.inject_key(KeyCode::Down, KeyMod::CTRL.with_shift());
    run_cycles(app, backend, 1);
    let mut labels = Vec::new();
    // Dropdown items start at row 2 (row 0=top border, row 1=tab bar, row 2..=items)
    for y in 2..30 {
        let row = backend.row(y);
        // Stop at bottom border or empty
        if row.contains("/4") || row.trim().is_empty() {
            break;
        }
        // Extract label: skip border char and number prefix, find the alpha part
        for name in ["Alpha", "Beta", "Gamma", "Delta"] {
            if row.contains(name) && !labels.contains(&name.to_string()) {
                labels.push(name.to_string());
            }
        }
    }
    // Close dropdown
    backend.inject_key(KeyCode::Esc, KeyMod::NONE);
    run_cycles(app, backend, 1);
    labels
}

/// Get the active tab name from the tab bar (row 1 of the panel area).
fn active_tab(backend: &MockBackend) -> String {
    let row = backend.row(1);
    for name in ["Alpha", "Beta", "Gamma", "Delta"] {
        // Active tab in LRU has no subscript prefix — check for " Name" pattern
        if row.contains(name) {
            return name.to_string();
        }
    }
    String::new()
}

#[test]
fn initial_lru_order() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    // Active should be Alpha (last set_active in demo)
    assert!(backend.contains("Content of Alpha"), "Alpha is active");
    // LRU order: Alpha(active), Beta(₁), Gamma(₂), Delta(₃)
    let labels = dropdown_labels(&mut app, &mut backend);
    assert_eq!(labels, vec!["Alpha", "Beta", "Gamma", "Delta"]);
}

#[test]
fn alt_1_switches_to_most_recent() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    // M-1 should switch to Beta (most recent other)
    backend.inject_key(KeyCode::Char('1'), KeyMod::ALT);
    run_cycles(&mut app, &mut backend, 2);
    assert!(backend.contains("Content of Beta"), "M-1 switched to Beta");
    // Now LRU: Beta(active), Alpha(₁), Gamma(₂), Delta(₃)
    let labels = dropdown_labels(&mut app, &mut backend);
    assert_eq!(labels, vec!["Beta", "Alpha", "Gamma", "Delta"]);
}

#[test]
fn alt_1_toggles_between_two() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    // Initial: Alpha active
    // M-1 → Beta
    backend.inject_key(KeyCode::Char('1'), KeyMod::ALT);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Content of Beta"), "first M-1 → Beta");
    // M-1 → Alpha (toggle back)
    backend.inject_key(KeyCode::Char('1'), KeyMod::ALT);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Content of Alpha"), "second M-1 → Alpha");
    // M-1 → Beta again
    backend.inject_key(KeyCode::Char('1'), KeyMod::ALT);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Content of Beta"), "third M-1 → Beta");
}

#[test]
fn alt_2_cycles_three_tabs() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    // Initial: Alpha active. LRU: Alpha, Beta, Gamma, Delta
    // M-2 → Gamma (position 2 in non-active LRU list)
    backend.inject_key(KeyCode::Char('2'), KeyMod::ALT);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Content of Gamma"), "M-2 → Gamma");
    // Now LRU: Gamma(active), Alpha(₁), Beta(₂), Delta(₃)
    let labels = dropdown_labels(&mut app, &mut backend);
    assert_eq!(labels, vec!["Gamma", "Alpha", "Beta", "Delta"]);
    // M-2 → Beta (pos 2: Alpha=₁, Beta=₂)
    backend.inject_key(KeyCode::Char('2'), KeyMod::ALT);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Content of Beta"), "M-2 → Beta");
    // Now LRU: Beta(active), Gamma(₁), Alpha(₂), Delta(₃)
    let labels = dropdown_labels(&mut app, &mut backend);
    assert_eq!(labels, vec!["Beta", "Gamma", "Alpha", "Delta"]);
    // M-2 → Alpha
    backend.inject_key(KeyCode::Char('2'), KeyMod::ALT);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Content of Alpha"), "M-2 → Alpha");
    // LRU: Alpha(active), Beta(₁), Gamma(₂), Delta(₃)
    let labels = dropdown_labels(&mut app, &mut backend);
    assert_eq!(labels, vec!["Alpha", "Beta", "Gamma", "Delta"]);
}

#[test]
fn alt_3_cycles_four_tabs() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    // Initial: Alpha active. LRU: Alpha, Beta, Gamma, Delta
    // M-3 → Delta (position 3)
    backend.inject_key(KeyCode::Char('3'), KeyMod::ALT);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Content of Delta"), "M-3 → Delta");
    // LRU: Delta(active), Alpha(₁), Beta(₂), Gamma(₃)
    let labels = dropdown_labels(&mut app, &mut backend);
    assert_eq!(labels, vec!["Delta", "Alpha", "Beta", "Gamma"]);
    // M-3 → Gamma
    backend.inject_key(KeyCode::Char('3'), KeyMod::ALT);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Content of Gamma"), "M-3 → Gamma");
    // LRU: Gamma(active), Delta(₁), Alpha(₂), Beta(₃)
    let labels = dropdown_labels(&mut app, &mut backend);
    assert_eq!(labels, vec!["Gamma", "Delta", "Alpha", "Beta"]);
    // M-3 → Beta
    backend.inject_key(KeyCode::Char('3'), KeyMod::ALT);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Content of Beta"), "M-3 → Beta");
    // LRU: Beta(active), Gamma(₁), Delta(₂), Alpha(₃)
    let labels = dropdown_labels(&mut app, &mut backend);
    assert_eq!(labels, vec!["Beta", "Gamma", "Delta", "Alpha"]);
    // M-3 → Alpha
    backend.inject_key(KeyCode::Char('3'), KeyMod::ALT);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Content of Alpha"), "M-3 → Alpha");
    // LRU: Alpha(active), Beta(₁), Gamma(₂), Delta(₃)
    let labels = dropdown_labels(&mut app, &mut backend);
    assert_eq!(labels, vec!["Alpha", "Beta", "Gamma", "Delta"]);
}

#[test]
fn dropdown_selection_matches_display_order() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    // Switch to Gamma so LRU changes
    backend.inject_key(KeyCode::Char('2'), KeyMod::ALT);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Content of Gamma"), "now on Gamma");
    // LRU: Gamma(active), Alpha(₁), Beta(₂), Delta(₃)
    // Open dropdown and select item 2 (should be Beta by display order)
    backend.inject_key(KeyCode::Down, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);
    // Navigate down twice (past Gamma, past Alpha → Beta)
    backend.inject_key(KeyCode::Down, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    backend.inject_key(KeyCode::Down, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    backend.inject_key(KeyCode::Enter, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Content of Beta"), "dropdown selected Beta correctly");
}
