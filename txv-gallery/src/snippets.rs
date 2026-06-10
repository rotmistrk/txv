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
        10 => DROPDOWN_MENU,
        _ => "",
    }
}

const STATUS_BAR: &str = r#"// StatusBar: priority-based horizontal layout
let mut bar = StatusBar::new();

// Left-aligned, high priority
let help = KeyLabelView::new(
    KeyEvent::new(KeyCode::F(1), KeyMod::NONE),
    CM_HELP, "F1 Help",
);
bar.add(StatusSlot::new(Box::new(help))
    .priority(9));

// Right-aligned
let quit = KeyLabelView::new(
    KeyEvent::new(KeyCode::F(10), KeyMod::NONE),
    CM_QUIT, "F10 Quit",
);
bar.add(StatusSlot::new(Box::new(quit))
    .priority(9).gravity(Gravity::Right));

// Stretch fills remaining space
let msg = MessageView::new(5);
bar.add(StatusSlot::new(Box::new(msg))
    .priority(3).stretch(1));"#;

const INPUT_LINE: &str = r#"// InputLine with completion + change events
let il = InputLine::new()
    .with_command(CM_OK)
    .with_change_command(CM_SEARCH_CHANGED)
    .with_completer(Box::new(MyCompleter));

// Completer trait:
impl Completer for MyCompleter {
    fn complete(
        &self, input: &str, cursor: usize,
        visitor: &mut CompletionVisitor,
    ) -> Result<(), Box<dyn Error>> {
        for name in &self.items {
            if name.starts_with(&input[..cursor]) {
                visitor(&MyCompletion(name))?;
            }
        }
        Ok(())
    }
}
// Emits CM_SIDEKICK_SHOW for popup"#;

const MODAL_KEY: &str = r#"// ModalKey: idle label, expands on trigger
let mk = ModalKey::new("F2 Go", "Go to: ")
    .trigger_key(KeyEvent::new(
        KeyCode::F(2), KeyMod::NONE))
    .add_child(Box::new(InputLine::new()));

// Place in StatusBar:
bar.add(StatusSlot::new(Box::new(mk))
    .priority(8));

// F2 → expands, shows "Go to: " + InputLine
// Deactivates on Enter/Esc/timeout"#;

const FRAME: &str = r#"// Frame: box border around a child
let mut ta = TextArea::new();
ta.set_content("Content inside frame");

let mut frame = Frame::new(Box::new(ta));
frame.set_label(FrameLabel::Top, "Title");
frame.set_label(FrameLabel::Bottom, "Status");
// Positions: LeftTop, Top, RightTop,
//   LeftBottom, Bottom, RightBottom"#;

const LIST_VIEW: &str = r#"// ListView with custom ListData trait
struct FileList { items: Vec<String> }

impl ListData for FileList {
    fn len(&self) -> usize { self.items.len() }
    fn label(&self, i: usize) -> &str {
        &self.items[i]
    }
    fn style(&self, i: usize) -> Style {
        if self.items[i].ends_with('/') {
            Style::default().with_fg(Color::Ansi(4))
        } else {
            Style::default()
        }
    }
}

let lv = ListView::new(FileList { items });
// Up/Down/Home/End/PgUp/PgDn navigate
// Enter emits CM_OK with cursor index"#;

const TREE_TABLE_VIEW: &str = r#"// TreeTableView: tree + typed columns
impl TreeTableSource for MySource {
    fn visible_count(&self) -> usize { .. }
    fn label(&self, row: usize) -> &str { .. }
    fn depth(&self, row: usize) -> usize { .. }
    fn is_expandable(&self, row: usize) -> bool
    fn toggle(&mut self, row: usize) { .. }
    fn column_count(&self) -> usize { 2 }
    fn cell(&self, row: usize, col: usize) -> &str
    fn column_align(&self, col: usize) -> ColAlign {
        ColAlign::Right // for numeric cols
    }
}
let ttv = TreeTableView::new(src, &[12, 10]);"#;

const SPLIT_PANE: &str = r#"// SplitPane: two views with resizable divider
let sp = SplitPane::new(
    SplitDirection::Horizontal,
    Box::new(left_view),
    Box::new(right_view),
);
sp.set_ratio(0.3); // 30%/70% split
sp.resize(5);      // grow left by 5 cols
// Also: SplitDirection::Vertical"#;

const TAB_PANEL: &str = r#"// TabPanel: tabbed container
let mut tp = TabPanel::new(TabBarMode::Static);
tp.insert_tab("Files", Box::new(file_list));
tp.insert_tab("Git", Box::new(git_view));
tp.insert_tab("Shell", Box::new(terminal));

// Modes: Static (all visible, numbered)
//   Lru (active + recent), Single (1 tab)
tp.set_active(0);
tp.tab_next();  // Alt+1..9 by position"#;

const FOCUS_GATED_GROUP: &str = r#"// FocusGatedGroup: context-sensitive keys
let mut fgg = FocusGatedGroup::new(1);
fgg.add_child(Box::new(KeyLabelView::new(
    KeyEvent::new(KeyCode::Char('a'), KeyMod::NONE),
    CM_ACTION, "a Action",
)));
fgg.add_child(Box::new(KeyLabelView::new(
    KeyEvent::new(KeyCode::Char('d'), KeyMod::NONE),
    CM_DELETE, "d Delete",
)));

// Dormant: width=0, invisible
// CM_ACTIVATE_GROUP(1) → shows keys
// CM_DEACTIVATE_GROUP(1) → hides"#;

const EDITOR: &str = r#"// EditorView: vi editor (uses GroupState)
let mut ev = EditorView::from_text(src);
ev.set_content(src, "rs"); // syntax hl

// : mode spawns InputLine (full readline)
// / and ? for incremental search
// Commands:
//   :set nu/nonu/wrap/nowrap/ai/noai
//   :set et/noet sw=4 ts=8 paste/nopaste
//   :set hls/nohls guides matchparen
//   :%!sort  :'<,'>!nl  :s/old/new/g
// Search: /pat ?pat n N * #
// Visual: v V C-v  Yank: yy p P
// Undo: u  Redo: C-r
// Software cursor (fg/bg flip)"#;

const DROPDOWN_MENU: &str = r#"// DropdownMenu: filterable popup list
let dd = DropdownMenu::new(source)
    .with_numbers(true)   // 1-9 hotkeys
    .with_filter(true)    // type to filter
    .with_max_visible(12)
    .with_open_side(OpenSide::Top);

// Try: type to filter, Up/Down, Enter/Esc
// Numbers 1-9 select directly
// Emits: CM_DROPDOWN_DONE (original idx)
//        CM_DROPDOWN_CANCELLED (Esc)"#;
