//! DropdownSource for tab panel dropdown.

use txv_core::prelude::*;

use super::tab_entry::TabEntry;
use crate::dropdown_source::DropdownSource;

/// Source for the tab dropdown menu.
pub(crate) struct TabDropdownSource {
    entries: Vec<TabEntry>,
}

impl TabDropdownSource {
    pub(crate) fn from_parts(
        titles: &[String],
        dirty: &[bool],
        badges: &[Option<String>],
        badge_styles: &[Option<Style>],
    ) -> Self {
        let entries = titles
            .iter()
            .enumerate()
            .map(|(i, t)| TabEntry {
                label: t.clone(),
                dirty: *dirty.get(i).unwrap_or(&false),
                badge: badges.get(i).and_then(|b| b.clone()),
                badge_style: badge_styles.get(i).and_then(|s| *s),
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

    fn badge(&self, idx: usize) -> Option<(&str, Style)> {
        let entry = self.entries.get(idx)?;
        if let Some(ref badge_text) = entry.badge {
            let style = entry
                .badge_style
                .unwrap_or_else(|| Style::new(palette().style(StyleId::Dim).fg(), Color::Transparent));
            return Some((badge_text.as_str(), style));
        }
        if entry.dirty {
            let fg = palette().style(StyleId::StateWarning).fg();
            Some(("•", Style::new(fg, Color::Transparent)))
        } else {
            None
        }
    }
}
