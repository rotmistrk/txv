//! New palette trait — single lookup by StyleId.

use std::sync::Arc;

use crate::cell::Style;

use super::style_id::StyleId;

/// A palette maps StyleId → Style. Views receive a palette from their parent.
/// Derived palettes can override specific IDs (e.g., modal overrides StatusBar).
pub trait StylePalette: Send + Sync {
    fn style(&self, id: StyleId) -> Style;
}

/// A palette that wraps another and overrides specific style IDs.
pub struct DerivedPalette {
    base: Arc<dyn StylePalette>,
    overrides: Vec<(StyleId, Style)>,
}

impl DerivedPalette {
    pub fn new(base: Arc<dyn StylePalette>) -> Self {
        Self {
            base,
            overrides: Vec::new(),
        }
    }

    pub fn with_override(mut self, id: StyleId, style: Style) -> Self {
        self.overrides.push((id, style));
        self
    }
}

impl StylePalette for DerivedPalette {
    fn style(&self, id: StyleId) -> Style {
        for &(oid, ref s) in &self.overrides {
            if oid == id {
                return *s;
            }
        }
        self.base.style(id)
    }
}
