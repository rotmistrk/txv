//! Types for TiledWorkspace panel configuration and layout.

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
    /// Human-readable name (for debugging/state persistence).
    pub name: String,
    /// Whether users can close tabs in this panel.
    pub closeable: bool,
    /// Whether the panel can be hidden by the user.
    pub hideable: bool,
    /// Whether the panel supports internal subpanel splitting.
    pub splittable: bool,
    /// Preferred position.
    pub position: PanelPosition,
    /// Tab bar mode for this panel.
    pub tab_mode: crate::tab_bar::TabBarMode,
}

impl PanelConfig {
    pub fn new(name: impl Into<String>, position: PanelPosition) -> Self {
        Self {
            name: name.into(),
            closeable: true,
            hideable: true,
            splittable: false,
            position,
            tab_mode: crate::tab_bar::TabBarMode::Lru,
        }
    }

    pub fn fixed(name: impl Into<String>, position: PanelPosition) -> Self {
        Self {
            name: name.into(),
            closeable: false,
            hideable: true,
            splittable: false,
            position,
            tab_mode: crate::tab_bar::TabBarMode::Static,
        }
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

/// Serializable workspace state for save/restore.
#[derive(Clone, Debug)]
pub struct WorkspaceState {
    /// Proportions for wide layout (flattened depth-first).
    pub wide_proportions: Vec<f32>,
    /// Proportions for narrow layout (flattened depth-first).
    pub narrow_proportions: Vec<f32>,
    /// Which panels are currently hidden.
    pub hidden: Vec<PanelId>,
}
