//! Demo widget factories — one per gallery entry.

use txv_core::prelude::*;
use txv_widgets::frame::{Frame, FrameLabel};
use txv_widgets::input_line::InputLine;
use txv_widgets::list_view::ListView;
use txv_widgets::split_pane::{SplitDirection, SplitPane};
use txv_widgets::tab_panel::TabPanel;
use txv_widgets::text_area::TextArea;
use txv_widgets::tree_table_view::TreeTableView;

use txv_widgets::v_key_label::KeyLabelView;

use crate::widget_list::WidgetListData;

mod demo_completer;
mod demo_completion;
mod demo_row;
mod demo_tree_source;
mod dropdown_demo;
mod image_chart_demo;
mod status_bar_demo;

use demo_completer::DemoCompleter;
pub(crate) use demo_tree_source::DemoTreeSource;

/// Create the demo widget for a given index.
pub(crate) fn make_demo(index: usize) -> Box<dyn View> {
    match index {
        0 => status_bar_demo::make(),
        1 => make_input_line(),
        2 => make_modal_key(),
        3 => make_frame(),
        4 => make_list_view(),
        5 => make_tree_table_view(),
        6 => make_split_pane(),
        7 => make_tab_panel(),
        8 => make_focus_gated_group(),
        9 => make_editor(),
        10 => make_dropdown(),
        11 => make_tab_dropdown(),
        12 => make_tab_lru(),
        13 => image_chart_demo::make(),
        _ => make_placeholder(),
    }
}

fn make_input_line() -> Box<dyn View> {
    let mut il = InputLine::new();
    il.set_text("Type here to test InputLine...");
    il.set_completer(Box::new(DemoCompleter));
    Box::new(il)
}

fn make_modal_key() -> Box<dyn View> {
    // ModalKey is a status-bar widget; demo it inside a StatusBar.
    use txv_core::status_bar::{StatusBar, StatusSlot};
    use txv_widgets::modal_key::ModalKey;
    let mk = ModalKey::new("F2 Go", "Go to: ")
        .trigger_key(KeyEvent::new(KeyCode::F(2), KeyMod::NONE))
        .add_child(Box::new(InputLine::new()));
    let mut bar = StatusBar::new();
    bar.add(StatusSlot::new(Box::new(mk)).priority(9).stretch(1));
    Box::new(bar)
}

fn make_frame() -> Box<dyn View> {
    let mut ta = TextArea::new();
    ta.set_content("Frame content here.\nLine 2.\nLine 3.");
    ta.show_line_numbers(false);
    let mut frame = Frame::new(Box::new(ta));
    frame.set_label(FrameLabel::Top, "Demo Frame");
    Box::new(frame)
}

fn make_list_view() -> Box<dyn View> {
    Box::new(ListView::new(WidgetListData))
}

fn make_tree_table_view() -> Box<dyn View> {
    let source = DemoTreeSource::new();
    Box::new(TreeTableView::new(source, &[12, 10]))
}

fn make_split_pane() -> Box<dyn View> {
    let mut left = TextArea::new();
    left.set_content("Left pane");
    left.show_line_numbers(false);
    let mut right = TextArea::new();
    right.set_content("Right pane");
    right.show_line_numbers(false);
    Box::new(SplitPane::new(
        SplitDirection::Horizontal,
        Box::new(left),
        Box::new(right),
    ))
}

fn make_tab_panel() -> Box<dyn View> {
    use txv_widgets::tab_bar::TabBarMode;
    let mut tp = TabPanel::new(TabBarMode::Static);
    let mut t1 = TextArea::new();
    t1.set_content("Tab 1 content");
    t1.show_line_numbers(false);
    tp.insert_tab("Alpha", Box::new(t1));
    let mut t2 = TextArea::new();
    t2.set_content("Tab 2 content");
    t2.show_line_numbers(false);
    tp.insert_tab("Beta", Box::new(t2));
    Box::new(tp)
}

fn make_focus_gated_group() -> Box<dyn View> {
    use txv_widgets::focus_gated_group::FocusGatedGroup;
    let mut fgg = FocusGatedGroup::new(1);
    let kl = KeyLabelView::new(KeyEvent::new(KeyCode::Char('a'), KeyMod::NONE), 100, "a Action");
    fgg.add_child(Box::new(kl));
    // Wrap in a frame to give it visible area
    let mut frame = Frame::new(Box::new(fgg));
    frame.set_label(FrameLabel::Top, "FocusGated");
    Box::new(frame)
}

fn make_editor() -> Box<dyn View> {
    use txv_edit::view::EditorView;
    let sample = "\
fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn main() {
    for i in 0..10 {
        println!(\"fib({i}) = {}\", fibonacci(i));
    }
}
";
    let mut ev = EditorView::from_text(sample);
    ev.set_content(sample, "rs");
    Box::new(ev)
}

fn make_placeholder() -> Box<dyn View> {
    let mut ta = TextArea::new();
    ta.set_content("(no demo)");
    ta.show_line_numbers(false);
    Box::new(ta)
}

fn make_dropdown() -> Box<dyn View> {
    dropdown_demo::make()
}

fn make_tab_dropdown() -> Box<dyn View> {
    use txv_widgets::tab_bar::TabBarMode;
    let mut tp = TabPanel::new(TabBarMode::Static);
    let mut t1 = TextArea::new();
    t1.set_content("Main tab content here.");
    t1.show_line_numbers(false);
    tp.insert_tab("Main", Box::new(t1));
    let mut t2 = TextArea::new();
    t2.set_content("Second tab content.");
    t2.show_line_numbers(false);
    tp.insert_tab("Tests", Box::new(t2));
    let mut t3 = TextArea::new();
    t3.set_content("Third tab — modified.");
    t3.show_line_numbers(false);
    tp.insert_tab("Build", Box::new(t3));
    // Tab 0: no badge. Tab 1: 1-char green badge "✓". Tab 2: 2-char red badge "!!".
    tp.set_badge_styled(
        1,
        Some("✓".to_string()),
        Some(Style::new(Color::Ansi(2), Color::Transparent)),
    );
    tp.set_badge_styled(
        2,
        Some("!!".to_string()),
        Some(Style::new(Color::Ansi(1), Color::Transparent)),
    );
    tp.set_active(0);
    Box::new(tp)
}

fn make_tab_lru() -> Box<dyn View> {
    use txv_widgets::tab_bar::TabBarMode;
    let mut tp = TabPanel::new(TabBarMode::Lru);
    for name in ["Alpha", "Beta", "Gamma", "Delta"] {
        let mut ta = TextArea::new();
        ta.set_content(&format!("Content of {name}"));
        ta.show_line_numbers(false);
        tp.insert_tab(name, Box::new(ta));
    }
    // Visit tabs to seed LRU: Delta, Gamma, Beta, Alpha (Alpha most recent)
    tp.set_active(3); // Delta
    tp.set_active(2); // Gamma
    tp.set_active(1); // Beta
    tp.set_active(0); // Alpha (active)
    Box::new(tp)
}
