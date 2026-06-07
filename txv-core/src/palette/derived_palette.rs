//! DerivedPalette — a palette that wraps another and overrides specific style IDs.

use std::sync::Arc;

use crate::cell::Style;

use super::style_id::StyleId;
use super::Palette;

/// A palette that wraps another and overrides specific style IDs.
pub struct DerivedPalette {
    base: Arc<dyn Palette>,
    overrides: Vec<(StyleId, Style)>,
}

impl DerivedPalette {
    pub fn new(base: Arc<dyn Palette>) -> Self {
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

impl Palette for DerivedPalette {
    fn style(&self, id: StyleId) -> Style {
        for &(oid, ref s) in &self.overrides {
            if oid == id {
                return *s;
            }
        }
        self.base.style(id)
    }
}
