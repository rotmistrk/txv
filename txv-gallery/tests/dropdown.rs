//! DropdownMenu scenario tests.

use txv_core::prelude::*;
use txv_widgets::dropdown_menu::{
    DropdownMenu, OpenSide, CM_DROPDOWN_CANCELLED, CM_DROPDOWN_CHANGED, CM_DROPDOWN_DONE,
};
use txv_widgets::dropdown_source::DropdownSource;

/// Simple test source with filterable string items.
struct TestSource {
    items: Vec<String>,
    visible: Vec<usize>,
}

impl TestSource {
    fn new(items: &[&str]) -> Self {
        let items: Vec<String> = items.iter().map(|s| s.to_string()).collect();
        let visible = (0..items.len()).collect();
        Self { items, visible }
    }
}

impl DropdownSource for TestSource {
    fn len(&self) -> usize {
        self.items.len()
    }
    fn label(&self, idx: usize) -> &str {
        &self.items[idx]
    }
    fn filter(&mut self, query: &str) {
        self.visible = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, s)| s.to_lowercase().contains(&query.to_lowercase()))
            .map(|(i, _)| i)
            .collect();
    }
    fn visible_len(&self) -> usize {
        self.visible.len()
    }
    fn visible_index(&self, visible_idx: usize) -> usize {
        self.visible[visible_idx]
    }
}

fn make_dropdown(items: &[&str]) -> (DropdownMenu<TestSource>, EventSink) {
    let source = TestSource::new(items);
    let mut dd = DropdownMenu::new(source);
    dd.set_bounds(Rect::new(0, 0, 30, 8));
    let sink = EventSink::new();
    dd.set_sink(sink.clone());
    (dd, sink)
}

fn key(dd: &mut DropdownMenu<TestSource>, code: KeyCode) {
    dd.handle(&Event::Key(KeyEvent::new(code, KeyMod::NONE)));
}

// ===== Basic =====

#[test]
fn renders_items() {
    let (mut dd, _) = make_dropdown(&["alpha", "beta", "gamma", "delta", "epsilon"]);
    dd.draw();
    let buf = dd.buffer();
    let row1 = (0..buf.width()).map(|x| buf.cell(x, 1).ch()).collect::<String>();
    assert!(row1.contains("alpha"), "first item visible: {row1}");
}

#[test]
fn cursor_moves_down() {
    let (mut dd, sink) = make_dropdown(&["one", "two", "three"]);
    key(&mut dd, KeyCode::Down);
    assert_eq!(dd.cursor(), 1);
    let events = sink.drain();
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::Command { id, .. } if *id == CM_DROPDOWN_CHANGED)));
}

#[test]
fn cursor_moves_up() {
    let (mut dd, _) = make_dropdown(&["one", "two", "three"]);
    key(&mut dd, KeyCode::Down);
    key(&mut dd, KeyCode::Down);
    key(&mut dd, KeyCode::Up);
    assert_eq!(dd.cursor(), 1);
}

#[test]
fn enter_emits_done_with_index() {
    let (mut dd, sink) = make_dropdown(&["apple", "banana", "cherry"]);
    key(&mut dd, KeyCode::Down); // cursor=1
    key(&mut dd, KeyCode::Enter);
    let events = sink.drain();
    let done = events
        .iter()
        .find(|e| matches!(e, Event::Command { id, .. } if *id == CM_DROPDOWN_DONE));
    assert!(done.is_some(), "should emit CM_DROPDOWN_DONE");
    if let Some(Event::Command { data, .. }) = done {
        let idx = data.as_ref().and_then(|d| d.downcast_ref::<usize>());
        assert_eq!(idx, Some(&1), "should select original index 1");
    }
}

#[test]
fn esc_emits_cancelled() {
    let (mut dd, sink) = make_dropdown(&["one", "two"]);
    key(&mut dd, KeyCode::Esc);
    let events = sink.drain();
    assert!(events
        .iter()
        .any(|e| matches!(e, Event::Command { id, .. } if *id == CM_DROPDOWN_CANCELLED)));
}

// ===== Filter =====

#[test]
fn filter_narrows_list() {
    let (mut dd, _) = make_dropdown(&["apple", "apricot", "banana", "blueberry"]);
    dd.handle(&Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyMod::NONE)));
    // "a" matches apple, apricot, banana (contains 'a')
    assert_eq!(dd.source().visible_len(), 3);
    dd.handle(&Event::Key(KeyEvent::new(KeyCode::Char('p'), KeyMod::NONE)));
    // "ap" matches apple, apricot
    assert_eq!(dd.source().visible_len(), 2);
}

#[test]
fn filter_resets_cursor() {
    let (mut dd, _) = make_dropdown(&["apple", "banana", "cherry"]);
    key(&mut dd, KeyCode::Down); // cursor=1
    dd.handle(&Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyMod::NONE)));
    assert_eq!(dd.cursor(), 0, "cursor resets after filter");
}

#[test]
fn backspace_widens_filter() {
    let (mut dd, _) = make_dropdown(&["apple", "apricot", "banana"]);
    dd.handle(&Event::Key(KeyEvent::new(KeyCode::Char('a'), KeyMod::NONE)));
    dd.handle(&Event::Key(KeyEvent::new(KeyCode::Char('p'), KeyMod::NONE)));
    dd.handle(&Event::Key(KeyEvent::new(KeyCode::Char('r'), KeyMod::NONE)));
    assert_eq!(dd.source().visible_len(), 1); // apricot
    key(&mut dd, KeyCode::Backspace);
    assert_eq!(dd.source().visible_len(), 2); // apple, apricot
}

#[test]
fn enter_on_filtered_emits_original_index() {
    let (mut dd, sink) = make_dropdown(&["apple", "banana", "cherry"]);
    // Filter to "ch" → only cherry (original index 2)
    dd.handle(&Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyMod::NONE)));
    dd.handle(&Event::Key(KeyEvent::new(KeyCode::Char('h'), KeyMod::NONE)));
    assert_eq!(dd.source().visible_len(), 1);
    key(&mut dd, KeyCode::Enter);
    let events = sink.drain();
    if let Some(Event::Command { data, .. }) = events
        .iter()
        .find(|e| matches!(e, Event::Command { id, .. } if *id == CM_DROPDOWN_DONE))
    {
        let idx = data.as_ref().and_then(|d| d.downcast_ref::<usize>());
        assert_eq!(idx, Some(&2), "should emit original index 2 (cherry)");
    } else {
        panic!("expected CM_DROPDOWN_DONE");
    }
}

// ===== Numbers =====

#[test]
fn number_hotkey_selects() {
    let source = TestSource::new(&["one", "two", "three", "four"]);
    let mut dd = DropdownMenu::new(source).with_numbers(true);
    dd.set_bounds(Rect::new(0, 0, 30, 8));
    let sink = EventSink::new();
    dd.set_sink(sink.clone());
    // Press '3' → selects index 2
    dd.handle(&Event::Key(KeyEvent::new(KeyCode::Char('3'), KeyMod::NONE)));
    let events = sink.drain();
    if let Some(Event::Command { data, .. }) = events
        .iter()
        .find(|e| matches!(e, Event::Command { id, .. } if *id == CM_DROPDOWN_DONE))
    {
        let idx = data.as_ref().and_then(|d| d.downcast_ref::<usize>());
        assert_eq!(idx, Some(&2));
    } else {
        panic!("expected CM_DROPDOWN_DONE on number press");
    }
}

// ===== Frame =====

#[test]
fn open_side_top_has_no_top_border() {
    let source = TestSource::new(&["one", "two"]);
    let mut dd = DropdownMenu::new(source).with_open_side(OpenSide::Top);
    dd.set_bounds(Rect::new(0, 0, 20, 5));
    dd.draw();
    let buf = dd.buffer();
    // Row 0 should NOT have '┌' corner
    assert_ne!(buf.cell(0, 0).ch(), '┌', "no top-left corner with open top");
    // But bottom should have corner
    assert_eq!(buf.cell(0, 4).ch(), '└', "bottom border present");
}

#[test]
fn desired_size_respects_item_count() {
    let source = TestSource::new(&["a", "b", "c"]);
    let dd = DropdownMenu::new(source);
    let (_, h) = dd.desired_size(40, 20);
    // 3 items + 2 borders = 5
    assert_eq!(h, 5);
}
