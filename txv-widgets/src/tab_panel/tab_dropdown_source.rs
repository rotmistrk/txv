//! DropdownSource for tab panel dropdown.

use txv_core::prelude::*;

use crate::dropdown_source::DropdownSource;

/// Source for the tab dropdown menu.
pub(crate) struct TabDropdownSource {
    entries: Vec<TabEntry>,
}

struct TabEntry {
    label: String,
    dirty: bool,
}

impl TabDropdownSource {
    pub(crate) fn from_parts(titles: &[String], dirty: &[bool]) -> Self {
        let entries = titles
            .iter()
            .zip(dirty.iter().chain(std::iter::repeat(&false)))
            .map(|(t, d)| TabEntry {
                label: t.clone(),
                dirty: *d,
            })
            .collect();
        Self { entries }
    }
}

impl DropdownSource for TabDropdownSource {
    fn len(&self) -> usize {
        self.entries.len()
    }

    fn label(&self, idx: usize) -> &str {
        self.entries.get(idx).map(|e| e.label.as_str()).unwrap_or("")
    }

    fn badge(&self, idx: usize) -> Option<(char, Style)> {
        let entry = self.entries.get(idx)?;
        if entry.dirty {
            let fg = palette().style(StyleId::StateWarning).fg();
            Some(('●', Style::new(fg, Color::Transparent)))
        } else {
            None
        }
    }
}
