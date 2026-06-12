//! Gallery app — root view using TiledWorkspace for 3-column layout.

use txv_core::prelude::*;
use txv_core::status_bar::{StatusBar, StatusSlot};
use txv_widgets::list_view::ListView;
use txv_widgets::sidekick_manager::SidekickManager;
use txv_widgets::text_area::TextArea;
use txv_widgets::tiled_workspace::types::{PanelConfig, PanelPosition, SplitNode};
use txv_widgets::tiled_workspace::TiledWorkspace;
use txv_widgets::v_key_label::KeyLabelView;

use crate::demos;
use crate::snippets;
use crate::widget_list::WidgetListData;

/// Panel indices within the TiledWorkspace.
const PANEL_LIST: usize = 0;
const PANEL_DEMO: usize = 1;
const PANEL_CODE: usize = 2;

/// Gallery root view: TiledWorkspace + StatusBar.
pub struct Gallery {
    group: GroupState,
    current_demo: usize,
}

impl Gallery {
    fn new() -> Self {
        let mut group = GroupState::new(ViewOptions::default().with_focusable());

        // Build TiledWorkspace (child 0)
        let workspace = Self::make_workspace();
        group.insert(Box::new(workspace));

        // StatusBar (child 1)
        let bar = Self::make_status_bar();
        group.insert(Box::new(bar));

        // SidekickManager (child 2) — postprocess, draws over everything
        let sk = SidekickManager::new();
        group.insert(Box::new(sk));

        group.set_focused_index(0);
        group.select_focused();

        Self { group, current_demo: 0 }
    }

    fn make_workspace() -> TiledWorkspace {
        let configs = vec![
            PanelConfig::fixed("Widgets", PanelPosition::Left),
            PanelConfig::fixed("Demo", PanelPosition::Center),
            PanelConfig::fixed("Code", PanelPosition::Right),
        ];
        let layout = SplitNode::h(vec![
            (0.20, SplitNode::leaf(PANEL_LIST)),
            (0.45, SplitNode::leaf(PANEL_DEMO)),
            (0.35, SplitNode::leaf(PANEL_CODE)),
        ]);
        let narrow = SplitNode::v(vec![
            (0.15, SplitNode::leaf(PANEL_LIST)),
            (0.55, SplitNode::leaf(PANEL_DEMO)),
            (0.30, SplitNode::leaf(PANEL_CODE)),
        ]);
        let mut ws = TiledWorkspace::new(configs, layout, narrow, 100);

        // Left panel: widget list
        let list = ListView::new(WidgetListData);
        ws.insert_tab(PANEL_LIST, "Widgets", Box::new(list));

        // Center panel: first demo
        let demo = demos::make_demo(0);
        ws.insert_tab(PANEL_DEMO, "Demo", demo);

        // Right panel: code snippet
        let mut code = TextArea::new();
        code.set_content(snippets::snippet_for(0));
        code.show_line_numbers(false);
        ws.insert_tab(PANEL_CODE, "Code", Box::new(code));

        ws
    }

    fn make_status_bar() -> StatusBar {
        use txv_widgets::tiled_workspace::commands::{CM_TW_ACTIVATE_TAB, CM_TW_TAB_DROPDOWN};
        let mut bar = StatusBar::new();
        let kl = KeyLabelView::new(KeyEvent::new(KeyCode::Char('q'), KeyMod::NONE), CM_QUIT, "q Quit");
        bar.add(StatusSlot::new(Box::new(kl)).priority(9));
        // Tab dropdown: Alt-0 + macOS º
        let dd = KeyLabelView::new(
            KeyEvent::new(KeyCode::Char('0'), KeyMod::ALT),
            CM_TW_TAB_DROPDOWN,
            "M-0 ▾",
        );
        bar.add(StatusSlot::new(Box::new(dd)).priority(7));
        let dd_mac = KeyLabelView::new(KeyEvent::new(KeyCode::Char('º'), KeyMod::NONE), CM_TW_TAB_DROPDOWN, "");
        bar.add(StatusSlot::new(Box::new(dd_mac)).priority(1));
        // macOS Option chars for tab switching (these don't conflict with Alt handling)
        let mac_chars: &[char] = &['¡', '™', '£', '¢', '∞', '§', '¶', '•', 'ª'];
        for n in 1u16..=9 {
            let mac_kl = KeyLabelView::new(
                KeyEvent::new(KeyCode::Char(mac_chars[(n - 1) as usize]), KeyMod::NONE),
                CM_TW_ACTIVATE_TAB,
                "",
            )
            .with_data(n - 1);
            bar.add(StatusSlot::new(Box::new(mac_kl)).priority(1));
        }
        let nav = KeyLabelView::new(KeyEvent::new(KeyCode::F(0), KeyMod::NONE), 0, "↑↓ Navigate");
        bar.add(StatusSlot::new(Box::new(nav)).priority(5));
        let tab = KeyLabelView::new(KeyEvent::new(KeyCode::F(0), KeyMod::NONE), 0, "C-S-↓ Tabs");
        bar.add(StatusSlot::new(Box::new(tab)).priority(5));
        bar
    }

    fn layout(&mut self) {
        let b = self.group.bounds();
        if b.w() == 0 || b.h() == 0 {
            return;
        }
        let status_h: u16 = 1;
        let ws_h = b.h().saturating_sub(status_h);
        self.group.set_child_bounds(0, Rect::new(0, 0, b.w(), ws_h));
        self.group.set_child_bounds(1, Rect::new(0, ws_h, b.w(), status_h));
    }

    fn workspace_mut(&mut self) -> &mut TiledWorkspace {
        self.group
            .child_mut(0)
            .and_then(|v| v.as_any_mut())
            .and_then(|a| a.downcast_mut::<TiledWorkspace>())
            .unwrap_or_else(|| unreachable!())
    }

    fn list_cursor(&mut self) -> usize {
        let ws = self.workspace_mut();
        let panel = ws.panel_mut(PANEL_LIST);
        panel
            .and_then(|p| p.active_child_mut())
            .and_then(|v| v.as_any_mut())
            .and_then(|a| a.downcast_mut::<ListView<WidgetListData>>())
            .map_or(0, |lv| lv.cursor())
    }

    fn switch_demo(&mut self, index: usize) {
        if index == self.current_demo {
            return;
        }
        self.current_demo = index;

        let ws = self.workspace_mut();

        // Replace demo tab content
        if let Some(panel) = ws.panel_mut(PANEL_DEMO) {
            panel.remove_tab(0);
        }
        let demo = demos::make_demo(index);
        ws.insert_tab(PANEL_DEMO, "Demo", demo);

        // Update code snippet
        if let Some(panel) = ws.panel_mut(PANEL_CODE) {
            if let Some(child) = panel.active_child_mut() {
                if let Some(ta) = child.as_any_mut().and_then(|a| a.downcast_mut::<TextArea>()) {
                    ta.set_content(snippets::snippet_for(index));
                }
            }
        }
    }
}

impl View for Gallery {
    delegate_group_state!(group, override { set_bounds, draw, handle });

    fn set_bounds(&mut self, r: Rect) {
        self.group.set_bounds(r);
        self.layout();
    }

    fn draw(&mut self) {
        self.group.buffer_mut().fill(' ', Style::default());
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let result = self.group.dispatch(event);

        // Tab/BackTab cycle focus if not consumed by a child
        if result == HandleResult::Ignored {
            if let Event::Key(key) = event {
                if key.code() == KeyCode::Tab && !key.modifiers().ctrl() && !key.modifiers().alt() {
                    self.workspace_mut().focus_next_visible();
                    return HandleResult::Consumed;
                }
                if key.code() == KeyCode::BackTab {
                    self.workspace_mut().focus_prev_visible();
                    return HandleResult::Consumed;
                }
            }
        }

        // After dispatch, sync demo with list cursor
        let new_cursor = self.list_cursor();
        if new_cursor != self.current_demo {
            self.switch_demo(new_cursor);
        }

        result
    }
}

/// Build the gallery app root view.
pub fn build_app() -> Gallery {
    let mut app = Gallery::new();
    app.set_bounds(Rect::new(0, 0, 80, 24));
    app
}
