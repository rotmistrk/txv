//! WorkspaceState — serializable workspace state for save/restore.

use super::types::PanelId;

/// Serializable workspace state for save/restore.
#[derive(Clone, Debug)]
pub struct WorkspaceState {
    pub(crate) wide_proportions: Vec<f32>,
    pub(crate) narrow_proportions: Vec<f32>,
    pub(crate) hidden: Vec<PanelId>,
}

impl WorkspaceState {
    pub fn new(wide_proportions: Vec<f32>, narrow_proportions: Vec<f32>, hidden: Vec<PanelId>) -> Self {
        Self {
            wide_proportions,
            narrow_proportions,
            hidden,
        }
    }

    pub fn wide_proportions(&self) -> &[f32] {
        &self.wide_proportions
    }

    pub fn narrow_proportions(&self) -> &[f32] {
        &self.narrow_proportions
    }

    pub fn hidden(&self) -> &[PanelId] {
        &self.hidden
    }
}
