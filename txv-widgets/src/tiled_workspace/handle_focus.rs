//! Focus direction helpers for TiledWorkspace.

use super::TiledWorkspace;

impl TiledWorkspace {
    pub(super) fn focus_direction_zoomed(&mut self, dx: i16, dy: i16) {
        if dx > 0 || dy > 0 {
            self.focus_next_visible();
        } else {
            self.focus_prev_visible();
        }
        self.zoomed = Some(self.group.focused_index());
        self.sync_visibility();
        self.recompute_layout();
    }

    pub(super) fn focus_direction_normal(&mut self, dx: i16, dy: i16) {
        let current = self.group.focused_index();
        let visible: Vec<usize> = (0..self.configs.len())
            .filter(|&i| !self.hidden[i])
            .filter(|&i| {
                self.group
                    .child(i)
                    .map(|c| c.bounds().w() > 0 && c.bounds().h() > 0)
                    .unwrap_or(false)
            })
            .collect();
        if visible.len() <= 1 {
            return;
        }
        let pos = visible.iter().position(|&i| i == current).unwrap_or(0);
        let forward = dx > 0 || dy > 0;
        let next = if forward {
            visible[(pos + 1) % visible.len()]
        } else {
            visible[(pos + visible.len() - 1) % visible.len()]
        };
        self.group.switch_focus(next);
    }
}
