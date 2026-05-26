//! StatusBar traits and container.
//!
//! Items implement `ActiveItem` (event handling), `VisibleItem` (rendering),
//! or both. The `StatusBar` container lays them out and routes events.

mod view_impl;

use crate::event::Event;
use crate::view::{EventSink, HandleResult, ViewOptions, ViewState};

/// Item alignment on the status bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gravity {
    Left,
    Right,
}

/// An item that translates events into commands.
pub trait ActiveItem: Send {
    fn handle(&mut self, event: &Event, sink: &EventSink) -> HandleResult;
    /// Whether this item wants exclusive control of the status bar.
    fn is_exclusive(&self) -> bool {
        false
    }
}

/// An item that renders a label on the status bar.
pub trait VisibleItem: Send {
    fn label(&self) -> &str;
    fn gravity(&self) -> Gravity;
    /// Priority for layout (higher = shown first when space is tight). Default: 5.
    fn priority(&self) -> u8 {
        5
    }
    /// Minimum width in characters. Default: label length + 2 (padding).
    fn min_width(&self) -> u16 {
        let l = self.label().len() as u16;
        if l == 0 {
            0
        } else {
            l + 2
        }
    }
    /// Maximum width (0 = unbounded). Default: same as min_width (fixed).
    fn max_width(&self) -> u16 {
        self.min_width()
    }
    /// Stretch weight for distributing extra space (0 = fixed). Default: 0.
    fn stretch(&self) -> u16 {
        0
    }
    /// Style for rendering the label. Default is plain.
    fn style(&self) -> crate::cell::Style {
        crate::cell::Style::default()
    }
    /// Called on tick so the item can update its label.
    fn tick(&mut self) {}
}

/// Combined trait for items that are both active and visible.
pub trait StatusBarItem: ActiveItem + VisibleItem {}

/// Blanket impl: anything implementing both traits is a StatusBarItem.
impl<T: ActiveItem + VisibleItem> StatusBarItem for T {}

// --- Internal storage ---

enum ItemSlot {
    Full(Box<dyn StatusBarItem>),
    ActiveOnly(Box<dyn ActiveItem>),
    VisibleOnly(Box<dyn VisibleItem>),
}

/// Composable status bar container. Implements `View`.
pub struct StatusBar {
    items: Vec<ItemSlot>,
    exclusive: Option<usize>,
    state: ViewState,
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            exclusive: None,
            state: ViewState::new(ViewOptions {
                preprocess: true,
                focusable: false,
                ..ViewOptions::default()
            }),
        }
    }

    /// Add an item that is both active and visible.
    pub fn add(&mut self, item: impl StatusBarItem + 'static) {
        self.items.push(ItemSlot::Full(Box::new(item)));
        self.state.mark_dirty();
    }

    /// Add an item that handles events but has no visible label.
    pub fn add_active_only(&mut self, item: impl ActiveItem + 'static) {
        self.items.push(ItemSlot::ActiveOnly(Box::new(item)));
    }

    /// Add an item that displays a label but does not handle events.
    pub fn add_visible_only(&mut self, item: impl VisibleItem + 'static) {
        self.items.push(ItemSlot::VisibleOnly(Box::new(item)));
        self.state.mark_dirty();
    }

    /// Put an item into exclusive mode (full-width rendering, sole event target).
    pub fn set_exclusive(&mut self, index: usize) {
        if index < self.items.len() {
            self.exclusive = Some(index);
            self.state.mark_dirty();
        }
    }

    /// Clear exclusive mode, returning to normal layout.
    pub fn clear_exclusive(&mut self) {
        self.exclusive = None;
        self.state.mark_dirty();
    }

    /// Whether an item is currently in exclusive mode.
    pub fn is_exclusive(&self) -> bool {
        self.exclusive.is_some()
    }

    fn tick_items(&mut self) {
        for slot in &mut self.items {
            match slot {
                ItemSlot::Full(item) => item.tick(),
                ItemSlot::VisibleOnly(item) => item.tick(),
                ItemSlot::ActiveOnly(_) => {}
            }
        }
        self.state.mark_dirty();
    }

    fn visible_label(&self, idx: usize) -> Option<&str> {
        match &self.items[idx] {
            ItemSlot::Full(item) => Some(item.label()),
            ItemSlot::VisibleOnly(item) => Some(item.label()),
            ItemSlot::ActiveOnly(_) => None,
        }
    }

    fn handle_active(&mut self, idx: usize, event: &Event, sink: &EventSink) -> HandleResult {
        match &mut self.items[idx] {
            ItemSlot::Full(item) => item.handle(event, sink),
            ItemSlot::ActiveOnly(item) => item.handle(event, sink),
            ItemSlot::VisibleOnly(_) => HandleResult::Ignored,
        }
    }

    fn item_is_exclusive(&self, idx: usize) -> bool {
        match &self.items[idx] {
            ItemSlot::Full(item) => item.is_exclusive(),
            ItemSlot::ActiveOnly(item) => item.is_exclusive(),
            ItemSlot::VisibleOnly(_) => false,
        }
    }

    fn item_priority(&self, idx: usize) -> u8 {
        match &self.items[idx] {
            ItemSlot::Full(item) => item.priority(),
            ItemSlot::VisibleOnly(item) => item.priority(),
            ItemSlot::ActiveOnly(_) => 0,
        }
    }

    fn item_gravity(&self, idx: usize) -> Gravity {
        match &self.items[idx] {
            ItemSlot::Full(item) => item.gravity(),
            ItemSlot::VisibleOnly(item) => item.gravity(),
            ItemSlot::ActiveOnly(_) => Gravity::Left,
        }
    }

    #[allow(dead_code)]
    fn item_min_width(&self, idx: usize) -> u16 {
        match &self.items[idx] {
            ItemSlot::Full(item) => item.min_width(),
            ItemSlot::VisibleOnly(item) => item.min_width(),
            ItemSlot::ActiveOnly(_) => 0,
        }
    }

    fn item_max_width(&self, idx: usize) -> u16 {
        match &self.items[idx] {
            ItemSlot::Full(item) => item.max_width(),
            ItemSlot::VisibleOnly(item) => item.max_width(),
            ItemSlot::ActiveOnly(_) => 0,
        }
    }

    fn item_stretch(&self, idx: usize) -> u16 {
        match &self.items[idx] {
            ItemSlot::Full(item) => item.stretch(),
            ItemSlot::VisibleOnly(item) => item.stretch(),
            ItemSlot::ActiveOnly(_) => 0,
        }
    }

    fn item_style(&self, idx: usize) -> crate::cell::Style {
        match &self.items[idx] {
            ItemSlot::Full(item) => item.style(),
            ItemSlot::VisibleOnly(item) => item.style(),
            ItemSlot::ActiveOnly(_) => crate::cell::Style::default(),
        }
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}
