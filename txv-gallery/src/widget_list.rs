//! Widget name list data for the left panel.

use txv_core::prelude::*;
use txv_widgets::ListData;

/// The names of widgets demonstrated in the gallery.
pub(crate) const WIDGET_NAMES: &[&str] = &[
    "StatusBar",
    "InputLine",
    "ModalKey",
    "Frame",
    "ListView",
    "TreeTableView",
    "SplitPane",
    "TabPanel",
    "FocusGatedGroup",
    "Editor",
    "DropdownMenu",
    "TabDropdown",
    "TabLRU",
];

/// ListData implementation backed by a static list of widget names.
pub(crate) struct WidgetListData;

impl ListData for WidgetListData {
    fn len(&self) -> usize {
        WIDGET_NAMES.len()
    }

    fn label(&self, index: usize) -> &str {
        WIDGET_NAMES.get(index).unwrap_or(&"")
    }

    fn style(&self, _index: usize) -> Style {
        Style::default()
    }
}
