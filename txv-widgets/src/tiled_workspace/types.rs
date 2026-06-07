//! Types for TiledWorkspace panel configuration and layout.

use crate::tab_bar::TabBarMode;

/// Identifies a panel by index.
pub type PanelId = usize;

/// Preferred panel position in the workspace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PanelPosition {
    Left,
    Center,
    Right,
    Bottom,
}

/// Configuration for a single panel.
#[derive(Clone, Debug)]
pub struct PanelConfig {
    pub(crate) name: String,
    pub(crate) closeable: bool,
    pub(crate) hideable: bool,
    pub(crate) splittable: bool,
    pub(crate) position: PanelPosition,
    pub(crate) tab_mode: TabBarMode,
}

impl PanelConfig {
    pub fn new(name: impl Into<String>, position: PanelPosition) -> Self {
        Self {
            name: name.into(),
            closeable: true,
            hideable: true,
            splittable: false,
            position,
            tab_mode: TabBarMode::Lru,
        }
    }

    pub fn fixed(name: impl Into<String>, position: PanelPosition) -> Self {
        Self {
            name: name.into(),
            closeable: false,
            hideable: true,
            splittable: false,
            position,
            tab_mode: TabBarMode::Static,
        }
    }

    pub fn with_splittable(mut self) -> Self {
        self.splittable = true;
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn closeable(&self) -> bool {
        self.closeable
    }

    pub fn hideable(&self) -> bool {
        self.hideable
    }

    pub fn splittable(&self) -> bool {
        self.splittable
    }

    pub fn position(&self) -> PanelPosition {
        self.position
    }

    pub fn tab_mode(&self) -> TabBarMode {
        self.tab_mode
    }
}

/// Split direction.
pub use crate::split_panel::SplitDir;

/// Layout mode — how the workspace decides wide vs narrow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutMode {
    /// Automatically switch based on terminal width threshold.
    Auto,
    /// Force wide layout regardless of width.
    Wide,
    /// Force narrow layout regardless of width.
    Narrow,
}

/// Layout tree node — defines how panels are arranged.
#[derive(Clone, Debug)]
pub enum SplitNode {
    Leaf(PanelId),
    Split {
        direction: SplitDir,
        children: Vec<(f32, SplitNode)>,
    },
}

impl SplitNode {
    /// Create a horizontal split (children laid out left to right).
    pub fn h(children: Vec<(f32, SplitNode)>) -> Self {
        Self::Split {
            direction: SplitDir::Horizontal,
            children,
        }
    }

    /// Create a vertical split (children laid out top to bottom).
    pub fn v(children: Vec<(f32, SplitNode)>) -> Self {
        Self::Split {
            direction: SplitDir::Vertical,
            children,
        }
    }

    /// Create a leaf node.
    pub fn leaf(id: PanelId) -> Self {
        Self::Leaf(id)
    }

    /// Collect all panel IDs in this tree.
    pub fn panel_ids(&self) -> Vec<PanelId> {
        match self {
            Self::Leaf(id) => vec![*id],
            Self::Split { children, .. } => children.iter().flat_map(|(_, c)| c.panel_ids()).collect(),
        }
    }

    /// Get mutable proportions for direct children of this node.
    pub fn proportions_mut(&mut self) -> Option<&mut Vec<(f32, SplitNode)>> {
        match self {
            Self::Split { children, .. } => Some(children),
            Self::Leaf(_) => None,
        }
    }
}
