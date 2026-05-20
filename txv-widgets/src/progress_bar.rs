//! ProgressBar — determinate or indeterminate progress indicator.

use txv_core::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ProgressMode {
    Determinate,
    Indeterminate,
}

pub struct ProgressBar {
    state: ViewState,
    pub mode: ProgressMode,
    pub progress: f32, // 0.0..=1.0
    pub tick: u16,     // for indeterminate animation
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            state: ViewState::new(ViewOptions {
                focusable: false,
                ..ViewOptions::default()
            }),
            mode: ProgressMode::Determinate,
            progress: 0.0,
            tick: 0,
        }
    }

    pub fn set_progress(&mut self, p: f32) {
        self.progress = p.clamp(0.0, 1.0);
        self.state.mark_dirty();
    }

    pub fn advance_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        self.state.mark_dirty();
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl View for ProgressBar {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let filled_style = txv_core::palette::palette().chrome.status_bar.to_style();
        let empty_style = Style::default();
        let pg = txv_core::glyphs::glyphs().progress;

        match self.mode {
            ProgressMode::Determinate => {
                let filled = (self.progress * w as f32) as u16;
                for col in 0..w {
                    let (ch, style) = if col < filled {
                        (pg.filled, filled_style)
                    } else {
                        (pg.empty, empty_style)
                    };
                    self.state.buffer_mut().put(col, 0, ch, style);
                }
            }
            ProgressMode::Indeterminate => {
                let pos = self.tick % w;
                let width = 3.min(w);
                for col in 0..w {
                    let in_bar = col >= pos && col < pos + width;
                    let (ch, style) = if in_bar {
                        (pg.filled, filled_style)
                    } else {
                        (pg.empty, empty_style)
                    };
                    self.state.buffer_mut().put(col, 0, ch, style);
                }
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Tick = event {
            if self.mode == ProgressMode::Indeterminate {
                self.advance_tick();
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }
}
