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

/// DropdownMenu widget — a leaf View rendering a filterable list.
pub struct DropdownMenu<D: DropdownSource> {
    pub(crate) state: ViewState,
    pub(crate) source: D,
    pub(crate) cursor: usize,
    pub(crate) scroll: ScrollView,
    pub(crate) filter: String,
    pub(crate) filter_enabled: bool,
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
            number_mode: NumberMode::None,
            max_visible: 12,
            open_side: OpenSide::None,
        }
    }

    pub fn with_filter(mut self, enabled: bool) -> Self {
        self.filter_enabled = enabled;
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

    pub(crate) fn content_height(&self) -> u16 {
        let b = self.state.bounds();
        let border = match self.open_side {
            OpenSide::None => 2,
            _ => 1,
        };
        b.h().saturating_sub(border)
    }

    fn sync_scroll(&mut self) {
        let h = self.content_height() as usize;
        self.scroll.set_viewport(h);
        self.scroll.set_total(self.source.visible_len());
        self.scroll.ensure_visible(self.cursor);
    }

    fn move_cursor(&mut self, delta: i32) -> HandleResult {
        let len = self.source.visible_len();
        if len == 0 {
            return HandleResult::Consumed;
        }
        let new = if delta < 0 {
            self.cursor.saturating_sub((-delta) as usize)
        } else {
            (self.cursor + delta as usize).min(len - 1)
        };
        if new != self.cursor {
            self.cursor = new;
            self.state.mark_dirty();
            self.state.put_command(CM_DROPDOWN_CHANGED, Some(Box::new(new)));
        }
        HandleResult::Consumed
    }

    fn select_current(&mut self) -> HandleResult {
        if self.source.visible_len() == 0 {
            return HandleResult::Consumed;
        }
        let orig = self.source.visible_index(self.cursor);
        self.state.put_command(CM_DROPDOWN_DONE, Some(Box::new(orig)));
        HandleResult::Consumed
    }

    fn handle_char(&mut self, ch: char) -> HandleResult {
        if self.number_mode != NumberMode::None && ch.is_ascii_digit() && ch != '0' {
            let n = (ch as usize) - ('1' as usize);
            let effective = match self.number_mode {
                NumberMode::SkipFirst => n + 1,
                _ => n,
            };
            if effective < self.source.visible_len() {
                let orig = self.source.visible_index(effective);
                self.state.put_command(CM_DROPDOWN_DONE, Some(Box::new(orig)));
            }
            return HandleResult::Consumed;
        }
        if self.filter_enabled && !ch.is_control() {
            self.filter.push(ch);
            self.source.filter(&self.filter);
            self.cursor = 0;
            self.state.mark_dirty();
            return HandleResult::Consumed;
        }
        HandleResult::Ignored
    }

    fn handle_backspace(&mut self) -> HandleResult {
        if self.filter.pop().is_some() {
            self.source.filter(&self.filter);
            self.cursor = 0;
            self.state.mark_dirty();
        }
        HandleResult::Consumed
    }

    fn autocomplete_lcp(&mut self) -> HandleResult {
        if !self.filter_enabled {
            return HandleResult::Ignored;
        }
        let count = self.source.visible_len();
        if count == 0 {
            return HandleResult::Consumed;
        }
        // Compute LCP of all visible labels
        let first = self.source.label(self.source.visible_index(0)).to_string();
        let mut lcp_len = first.len();
        for i in 1..count {
            let label = self.source.label(self.source.visible_index(i));
            lcp_len = first
                .chars()
                .zip(label.chars())
                .take(lcp_len)
                .take_while(|(a, b)| a.to_lowercase().eq(b.to_lowercase()))
                .count();
            if lcp_len == 0 {
                break;
            }
        }
        let lcp: String = first.chars().take(lcp_len).collect();
        if lcp.len() > self.filter.len() {
            self.filter = lcp.to_lowercase();
            self.source.filter(&self.filter);
            self.cursor = 0;
            self.state.mark_dirty();
        }
        HandleResult::Consumed
    }
}

impl<D: DropdownSource> View for DropdownMenu<D> {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w < 3 || h < 2 {
            return;
        }
        self.sync_scroll();
        let pal = palette();
        let bg = pal.style(StyleId::PopupBackground);
        let border_style = pal.style(StyleId::Border);
        let selected = pal.style(StyleId::CursorFocused);
        let dim = pal.style(StyleId::Dim);
        self.state.buffer_mut().fill(' ', bg);
        self.draw_frame(w, h, border_style);
        self.draw_items(w, bg, selected, dim);
        self.draw_filter_label(w, h, border_style);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        match key.code() {
            KeyCode::Up | KeyCode::Char('k') => self.move_cursor(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_cursor(1),
            KeyCode::Enter => self.select_current(),
            KeyCode::Esc => {
                self.state.put_command(CM_DROPDOWN_CANCELLED, None);
                HandleResult::Consumed
            }
            KeyCode::Tab | KeyCode::Right if self.filter_enabled => self.autocomplete_lcp(),
            KeyCode::Backspace if self.filter_enabled => self.handle_backspace(),
            KeyCode::Char(ch) => self.handle_char(ch),
            _ => HandleResult::Ignored,
        }
    }
}
