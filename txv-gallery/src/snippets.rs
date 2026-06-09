//! Source code snippets shown in the right panel for each demo widget.

/// Returns the setup snippet for the widget at the given index.
pub(crate) fn snippet_for(index: usize) -> &'static str {
    match index {
        0 => STATUS_BAR,
        1 => INPUT_LINE,
        2 => MODAL_KEY,
        3 => FRAME,
        4 => LIST_VIEW,
        5 => TREE_TABLE_VIEW,
        6 => SPLIT_PANE,
        7 => TAB_PANEL,
        8 => FOCUS_GATED_GROUP,
        9 => EDITOR,
        _ => "",
    }
}

const STATUS_BAR: &str = r#"let mut bar = StatusBar::new();
bar.add(StatusSlot::new(
    Box::new(KeyLabelView::new(
        KeyEvent::new(KeyCode::F(1), KeyMod::NONE),
        CM_HELP,
        "F1 Help",
    ))
).priority(9));
bar.set_bounds(Rect::new(0, 0, 80, 1));"#;

const INPUT_LINE: &str = r#"let mut il = InputLine::new();
il.set_text("type here...");
il.set_bounds(Rect::new(0, 0, 40, 1));"#;

const MODAL_KEY: &str = r#"let mut mk = ModalKey::new("F2 Go", "Go to: ");
mk.add_trigger(KeyEvent::new(
    KeyCode::F(2), KeyMod::NONE,
));
mk.add_child(Box::new(InputLine::new()));
mk.set_bounds(Rect::new(0, 0, 20, 1));"#;

const FRAME: &str = r#"let child = Box::new(TextArea::new());
let mut frame = Frame::new(child);
frame.set_label(FrameLabel::Top, "Title");
frame.set_bounds(Rect::new(0, 0, 40, 10));"#;

const LIST_VIEW: &str = r#"let data = MyListData::new(items);
let mut lv = ListView::new(data);
lv.set_bounds(Rect::new(0, 0, 30, 10));
// Arrow keys navigate, Enter emits CM_OK"#;

const TREE_TABLE_VIEW: &str = r#"let source = MyTreeSource::new(rows);
let mut ttv = TreeTableView::new(source);
ttv.set_bounds(Rect::new(0, 0, 60, 15));"#;

const SPLIT_PANE: &str = r#"let sp = SplitPane::new(
    SplitDirection::Horizontal,
    Box::new(left_view),
    Box::new(right_view),
);
sp.set_ratio(0.3);"#;

const TAB_PANEL: &str = r#"let mut tp = TabPanel::new();
tp.add_tab("Tab 1", Box::new(view1));
tp.add_tab("Tab 2", Box::new(view2));
tp.set_bounds(Rect::new(0, 0, 60, 20));"#;

const FOCUS_GATED_GROUP: &str = r#"let mut fgg = FocusGatedGroup::new(1);
fgg.add_child(Box::new(KeyLabelView::new(
    KeyEvent::new(KeyCode::Char('a'), KeyMod::NONE),
    CM_ACTION,
    "a Action",
)));
// Activate with CM_ACTIVATE_GROUP(1)"#;

const EDITOR: &str = r#"let mut ev = EditorView::from_text(src);
ev.set_content(src, "rs");
ev.set_bounds(Rect::new(0, 0, 60, 20));
// Vi keys: h/j/k/l, i/Esc, dd, yy/p
// :set nu, :set wrap, :set guides
// /pattern  - search forward (incremental)
// ?pattern  - search backward
// n/N       - next/prev match
// :s/old/new/g - substitute
// Visual: v, V, C-v"#;
