//! Dropdown overlay rendering for TabPanel.

use txv_core::prelude::*;

use super::TabPanel;

impl TabPanel {
    /// Draw the dropdown overlay into the panel's buffer (below row 0).
    pub(crate) fn draw_dropdown_overlay(&mut self) {
        let entries = self.bar.dropdown_entries();
        if entries.is_empty() {
            return;
        }
        let cursor = self.bar.dropdown_cursor.unwrap_or(0);
        let pal = txv_core::palette::palette();
        let normal = pal.chrome.bar.to_style();
        let selected = Style {
            fg: normal.bg,
            bg: normal.fg,
            ..Style::default()
        };

        let buf = self.state.buffer_mut();
        let max_rows = (buf.height().saturating_sub(1)) as usize;
        let visible = entries.len().min(max_rows);

        for (row, (_, label)) in entries.iter().take(visible).enumerate() {
            let y = (row as u16) + 1; // below tab bar
            let style = if row == cursor {
                selected
            } else {
                normal
            };
            // Clear the line area for the dropdown
            let w = buf.width().min(label.len() as u16 + 2);
            for x in 0..w {
                buf.put(x, y, ' ', style);
            }
            buf.print(1, y, label, style);
        }
    }
}
