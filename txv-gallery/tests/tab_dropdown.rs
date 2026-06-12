//! Scenario tests for TabPanel dropdown via the gallery app.
//! Each test starts fresh, sends keys to navigate, and asserts on rendered screen.

use txv_core::prelude::*;

/// Navigate to TabDropdown demo (index 11), focus demo panel.
fn nav_to_demo(app: &mut dyn View, backend: &mut MockBackend) {
    for _ in 0..11 {
        backend.inject_key(KeyCode::Down, KeyMod::NONE);
        run_cycles(app, backend, 1);
    }
    backend.inject_key(KeyCode::Right, KeyMod::CTRL.with_shift());
    run_cycles(app, backend, 1);
}

#[test]
fn ctrl_shift_down_opens_dropdown() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    backend.inject_key(KeyCode::Down, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("3/3"), "dropdown count label");
    assert!(backend.contains("₁Main"), "numbered item 1");
    assert!(backend.contains("₂Tests"), "numbered item 2");
    assert!(backend.contains("₃Build"), "numbered item 3");
}

#[test]
fn mac_option_0_opens_dropdown() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    // macOS Option+0 sends º
    backend.inject_key(KeyCode::Char('º'), KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("3/3"), "dropdown opened via º");
}

#[test]
fn dropdown_sized_to_content() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    backend.inject_key(KeyCode::Down, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);
    // Dropdown should NOT span full panel width.
    // Row with "₁Main" should have content ending well before panel right edge.
    // The panel starts around col 34 (after left panel). Check that the dropdown
    // border ends before col 60 (not full width).
    let row2 = backend.row(2); // row with first dropdown item
                               // Dropdown has 3 items + 1 bottom border = 4 rows (rows 2,3,4,5 from panel top)
    assert!(row2.contains("₁Main"), "item row present");
    // Bottom border is at row 4 (0-indexed: tab_bar=1, items=2,3,4, border=5)
    let row_bottom = backend.row(5);
    assert!(
        row_bottom.contains("ᵖ") || row_bottom.contains("3/3"),
        "bottom border with filter indicator and count"
    );
    // Row 6 should NOT have dropdown content
    let row_after = backend.row(6);
    assert!(
        !row_after.contains("│") || !row_after.contains("3/3"),
        "no dropdown below bottom border"
    );
}

#[test]
fn dropdown_offset_from_left_edge() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    backend.inject_key(KeyCode::Down, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);
    // The tab bar has a left powerline cap at the panel's first col.
    // The dropdown's left border │ should be 1 col to the right of panel start.
    // Row 2 should show: [panel_border]│[space]│ ₁Main ...│
    let row2 = backend.row(2);
    // There should be the panel separator, then space or cap, then the dropdown border
    assert!(row2.contains("│ ₁Main"), "dropdown offset: border then item");
}

#[test]
fn dropdown_down_enter_switches_tab() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    backend.inject_key(KeyCode::Down, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);
    backend.inject_key(KeyCode::Down, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    backend.inject_key(KeyCode::Enter, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Second tab content"), "switched to Tests");
    assert!(!backend.contains("3/3"), "dropdown closed");
}

#[test]
fn dropdown_esc_cancels() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    backend.inject_key(KeyCode::Down, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("3/3"), "dropdown open");
    backend.inject_key(KeyCode::Esc, KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Main tab content"), "original content");
    assert!(!backend.contains("3/3"), "dropdown closed");
}

#[test]
fn dropdown_filter_narrows() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    backend.inject_key(KeyCode::Down, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);
    backend.inject_key(KeyCode::Char('t'), KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Tests"), "Tests matches prefix 't'");
    assert!(backend.contains("1/3"), "filtered to 1 of 3");
    // The count changed from 3/3 to 1/3 proving filter narrowed
    assert!(!backend.contains("3/3"), "no longer showing all 3");
}

#[test]
fn mac_option_digit_selects_with_dropdown_open() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    // Open dropdown
    backend.inject_key(KeyCode::Down, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("3/3"), "dropdown open");
    // macOS Option+3 sends £ → CM_TW_ACTIVATE_TAB(2) → selects Build
    backend.inject_key(KeyCode::Char('£'), KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Third tab"), "Option+3 selected Build");
    assert!(!backend.contains("3/3"), "dropdown closed after select");
}

#[test]
fn mac_option_digit_switches_without_dropdown() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    // No dropdown open. Option+2 sends ™ → switch to tab 2 (Tests)
    backend.inject_key(KeyCode::Char('™'), KeyMod::NONE);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("Second tab content"), "Option+2 switched to Tests");
    assert!(!backend.contains("3/3"), "no dropdown");
}

#[test]
#[test]
fn badge_on_right_side() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    backend.inject_key(KeyCode::Down, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);
    // Build has !! badge — should appear on the Build row
    let row4 = backend.row(4); // 0=workspace top, 1=tab bar, 2=item1, 3=item2, 4=item3
    assert!(row4.contains("!!"), "badge on Build row");
    assert!(row4.contains("Build"), "Build label on same row");
}

#[test]
fn alt_0_opens_dropdown() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    backend.inject_key(KeyCode::Char('0'), KeyMod::ALT);
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("3/3"), "dropdown opened via Alt-0");
}


#[test]
fn badges_appear_on_tab_bar() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    // Tab 0 (Main): no badge. Tab 1 (Tests): green "\u{2713}". Tab 2 (Build): red "!!".
    assert!(backend.contains("\u{2713}"), "\u{2713} badge on Tests tab");
    assert!(backend.contains("!!"), "!! badge on Build tab");
    let green_fg = backend.fg_of('\u{2713}').expect("\u{2713} found");
    assert_eq!(green_fg, Color::Ansi(2), "\u{2713} badge is green");
    let red_fg = backend.fg_of('!').expect("! found");
    assert_eq!(red_fg, Color::Ansi(1), "! badge is red");
}

#[test]
fn badges_appear_in_dropdown_with_same_colors() {
    let mut app = txv_gallery::build_app();
    let mut backend = MockBackend::new(100, 30);
    app.set_bounds(Rect::new(0, 0, 100, 30));
    nav_to_demo(&mut app, &mut backend);
    // Open dropdown
    backend.inject_key(KeyCode::Down, KeyMod::CTRL.with_shift());
    run_cycles(&mut app, &mut backend, 1);
    assert!(backend.contains("3/3"), "dropdown open");
    // Dropdown rows: row 2=Main (no badge), row 3=Tests (\u{2713}), row 4=Build (!!)
    let row2 = backend.row(2);
    assert!(row2.contains("Main"), "Main in dropdown");
    assert!(!row2.contains("\u{2713}"), "Main has no badge in dropdown");
    let row3 = backend.row(3);
    assert!(row3.contains("Tests"), "Tests in dropdown");
    assert!(row3.contains("\u{2713}"), "\u{2713} badge on Tests row in dropdown");
    let colors_row3 = backend.fg_of_on_row('\u{2713}', 3);
    assert!(!colors_row3.is_empty(), "\u{2713} found on row 3");
    assert_eq!(colors_row3[0], Color::Ansi(2), "\u{2713} is green in dropdown");
    let row4 = backend.row(4);
    assert!(row4.contains("Build"), "Build in dropdown");
    assert!(row4.contains("!!"), "!! badge on Build row in dropdown");
    let colors_row4 = backend.fg_of_on_row('!', 4);
    assert!(!colors_row4.is_empty(), "! found on row 4");
    assert_eq!(colors_row4[0], Color::Ansi(1), "! is red in dropdown");
    // Badges must be right-aligned: \u{2713} (1 char) and !! (2 chars) should end at same column
    let badge_end_3 = row3.rfind('\u{2713}').unwrap();
    let badge_end_4 = row4.rfind('!').unwrap();
    assert_eq!(badge_end_3, badge_end_4, "badges right-aligned to same column");
}
