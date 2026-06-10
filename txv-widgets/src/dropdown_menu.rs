//! DropdownMenu — bordered filterable popup list widget.

use txv_core::event::CommandId;
use txv_core::prelude::*;

use crate::dropdown_source::DropdownSource;
use crate::scroll_view::ScrollView;

/// Emitted when cursor moves. Data: `Box<usize>` (visible index).
pub const CM_DROPDOWN_CHANGED: CommandId = 230;
/// Emitted on Enter. Data: `Box<usize>` (original index).
pub const CM_DROPDOWN_DONE: CommandId = 231;
/// Emitted on Esc.
pub const CM_DROPDOWN_CANCELLED: CommandId = 232;

/// Which side of the frame is open (no border).
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum OpenSide {
    #[default]
    None,
    Top,
    Bottom,
}

/// Numbering mode for dropdown items.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum NumberMode {
    #[default]
    None,
    /// All items numbered ₁₂₃...₉
    All,
    /// First item blank, rest numbered ₁₂₃...₈ (LRU tab bar style)
    SkipFirst,
}

/// Filter/search mode for the dropdown.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterMode {
    /// No filtering. Plain digits are hotkeys.
    None,
    /// Prefix match. Tab fills LCP.
    #[default]
    Prefix,
    /// Substring match (contains anywhere).
    Substring,
    /// Ordered subsequence (fuzzy).
    Subsequence,
}

/// DropdownMenu widget — a leaf View rendering a filterable list.
pub struct DropdownMenu<D: DropdownSource> {
    pub(crate) state: ViewState,
    pub(crate) source: D,
    pub(crate) cursor: usize,
    pub(crate) scroll: ScrollView,
    pub(crate) filter: String,
    pub(crate) filter_enabled: bool,
    pub(crate) filter_mode: FilterMode,
    pub(crate) number_mode: NumberMode,
    pub(crate) max_visible: usize,
    pub(crate) open_side: OpenSide,
}

impl<D: DropdownSource> DropdownMenu<D> {
    pub fn new(source: D) -> Self {
        Self {
            state: ViewState::new(ViewOptions::default().with_focusable()),
            source,
            cursor: 0,
            scroll: ScrollView::new(),
            filter: String::new(),
            filter_enabled: true,
            filter_mode: FilterMode::Prefix,
            number_mode: NumberMode::None,
            max_visible: 12,
            open_side: OpenSide::None,
        }
    }

    pub fn with_filter(mut self, mode: FilterMode) -> Self {
        self.filter_mode = mode;
        self.filter_enabled = mode != FilterMode::None;
        self
    }

    pub fn with_numbers(mut self, mode: NumberMode) -> Self {
        self.number_mode = mode;
        self
    }

    pub fn with_max_visible(mut self, n: usize) -> Self {
        self.max_visible = n;
        self
    }

    pub fn with_open_side(mut self, side: OpenSide) -> Self {
        self.open_side = side;
        self
    }

    pub fn desired_size(&self, max_w: u16, max_h: u16) -> (u16, u16) {
        let count = self.source.visible_len().min(self.max_visible);
        let border_h: u16 = match self.open_side {
            OpenSide::None => 2,
            _ => 1,
        };
        let h = (count as u16 + border_h).min(max_h);
        (max_w, h)
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn source(&self) -> &D {
        &self.source
    }

    pub fn source_mut(&mut self) -> &mut D {
        &mut self.source
    }

    pub fn filter_mode(&self) -> FilterMode {
        self.filter_mode
    }

    pub(crate) fn content_height(&self) -> u16 {
        let b = self.state.bounds();
        let border = match self.open_side {
            OpenSide::None => 2,
            _ => 1,
        };
        b.h().saturating_sub(border)
    }

    pub(crate) fn sync_scroll(&mut self) {
        let h = self.content_height() as usize;
        self.scroll.set_viewport(h);
        self.scroll.set_total(self.source.visible_len());
        self.scroll.ensure_visible(self.cursor);
    }
}
