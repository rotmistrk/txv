//! ScrollView — scroll state helper embedded by scrollable widgets.
//! Not a View itself.

/// Tracks scroll offset and viewport for a scrollable region.
pub struct ScrollView {
    pub(crate) offset: usize,
    pub(crate) total: usize,
    pub(crate) viewport: usize,
}

impl ScrollView {
    pub fn new() -> Self {
        Self {
            offset: 0,
            total: 0,
            viewport: 0,
        }
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn viewport(&self) -> usize {
        self.viewport
    }

    pub fn set_total(&mut self, total: usize) {
        self.total = total;
        self.clamp();
    }

    pub fn set_viewport(&mut self, viewport: usize) {
        self.viewport = viewport;
        self.clamp();
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.offset = self.offset.saturating_sub(n);
    }

    pub fn scroll_down(&mut self, n: usize) {
        self.offset = self.offset.saturating_add(n);
        self.clamp();
    }

    pub fn scroll_to(&mut self, pos: usize) {
        self.offset = pos;
        self.clamp();
    }

    /// Ensure the given index is visible, scrolling minimally.
    pub fn ensure_visible(&mut self, index: usize) {
        if index < self.offset {
            self.offset = index;
        } else if self.viewport > 0 && index >= self.offset + self.viewport {
            self.offset = index.saturating_sub(self.viewport.saturating_sub(1));
        }
    }

    pub fn max_offset(&self) -> usize {
        self.total.saturating_sub(self.viewport)
    }

    /// Rows one PgUp/PgDn moves: a viewport minus one row of overlap.
    pub fn page_size(&self) -> usize {
        self.viewport.saturating_sub(1).max(1)
    }

    /// PgDn: move the viewport forward one page and return the cursor moved by
    /// the same amount, so it keeps its screen row. Clamped to the last item.
    pub fn page_down(&mut self, cursor: usize) -> usize {
        let step = self.page_size();
        self.scroll_down(step);
        cursor.saturating_add(step).min(self.total.saturating_sub(1))
    }

    /// PgUp: move the viewport back one page and return the cursor moved by
    /// the same amount. Clamped to the first item.
    pub fn page_up(&mut self, cursor: usize) -> usize {
        let step = self.page_size();
        self.scroll_up(step);
        cursor.saturating_sub(step)
    }

    /// Thumb position and size for a scrollbar (0-based row, height).
    pub fn thumb(&self, track_height: u16) -> (u16, u16) {
        if self.total == 0 || self.viewport >= self.total {
            return (0, track_height);
        }
        let th = track_height as usize;
        let size = (self.viewport * th / self.total).max(1);
        let max_off = self.max_offset();
        let pos = if max_off == 0 {
            0
        } else {
            self.offset * (th.saturating_sub(size)) / max_off
        };
        (pos as u16, size as u16)
    }

    fn clamp(&mut self) {
        let max = self.max_offset();
        if self.offset > max {
            self.offset = max;
        }
    }
}

impl Default for ScrollView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "scroll_view_tests.rs"]
mod tests;
