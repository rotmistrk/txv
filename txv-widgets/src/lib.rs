//! # txv-widgets
//!
//! Concrete View implementations — ready-to-use interactive TUI components.
//! Depends only on txv-core (plus `ignore` for filesystem widgets).

pub mod command_item;
pub mod confirm_item;
pub mod dialog;
pub mod file_list;
pub mod file_tree;
mod file_tree_filter;
pub mod focus_gated_group;
pub mod frame;
pub mod fuzzy_select;
pub mod input_dialog;
pub mod input_line;
pub mod key_encode;
pub mod list_view;
pub mod menu;
pub mod modal_key;
pub mod prefix_item;
pub mod progress_bar;
pub mod pty_session;
pub mod pty_terminal;
mod pty_terminal_scroll;
mod pty_terminal_spawn;
pub mod scroll_view;
pub mod scrollbar;
pub mod sidekick;
pub mod sidekick_manager;
pub mod split_pane;
pub mod split_panel;
pub mod status_bar;
pub mod status_indicators;
pub mod status_items;
#[path = "tab_bar_new/mod.rs"]
pub mod tab_bar;
pub mod tab_panel;
pub mod table;
pub mod text_area;
pub mod tiled_workspace;
pub mod tree_view;
pub mod v_branch;
pub mod v_clock;
pub mod v_command_line;
pub mod v_confirm;
pub mod v_key_label;
pub mod v_message;
pub mod v_prefix;

pub(crate) mod prefix_binding;

pub use command_item::CommandItem;
pub use dialog::Dialog;
pub use file_list::FileListData;
pub use file_tree::FileTreeData;
pub use file_tree_filter::fuzzy_match_positions;
pub use focus_gated_group::{FocusGatedGroup, CM_ACTIVATE_GROUP, CM_DEACTIVATE_GROUP};
pub use fuzzy_select::FuzzySelect;
pub use input_dialog::InputDialog;
pub use input_line::InputLine;
pub use list_view::{ListData, ListView};
pub use menu::{Menu, MenuItem};
pub use modal_key::ModalKey;
pub use progress_bar::{ProgressBar, ProgressMode};
pub use pty_terminal::PtyTerminal;
pub use scroll_view::ScrollView;
pub use scrollbar::Scrollbar;
pub use split_pane::{SplitDirection, SplitPane};
pub use split_panel::{SplitDir, SplitPanel};
pub use status_bar::{StatusBar, StatusItem};
pub use status_indicators::{BranchItem, CursorPos, ModeItem, PositionItem};
pub use status_items::{ClockItem, KeyLabelItem, MessageItem, CM_STATUS_MESSAGE};
pub use tab_bar::{TabBar, TabBarMode};
pub use tab_panel::TabPanel;
pub use table::{Column, Table};
pub use text_area::TextArea;
pub use tree_view::{TreeData, TreeView};
pub use v_branch::BranchView;
pub use v_clock::ClockView;
pub use v_command_line::CommandLineView;
pub use v_confirm::ConfirmView;
pub use v_key_label::KeyLabelView;
pub use v_message::MessageView;
pub use v_prefix::PrefixView;

#[cfg(test)]
#[path = "palette_integration_tests.rs"]
mod palette_integration_tests;

#[cfg(test)]
#[path = "glyphs_integration_tests.rs"]
mod glyphs_integration_tests;

#[cfg(test)]
#[path = "cursor_integration_tests.rs"]
mod cursor_integration_tests;

#[cfg(test)]
#[path = "modal_key_tests.rs"]
mod modal_key_tests;
