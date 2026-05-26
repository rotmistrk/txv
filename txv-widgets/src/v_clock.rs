//! ClockView — shows current time, updates on tick.

use std::time::Instant;
use txv_core::prelude::*;

/// A View-based status bar item that displays the current time.
pub struct ClockView {
    state: ViewState,
    interval_secs: u16,
    last_update: Instant,
    label_text: String,
}

impl ClockView {
    pub fn new(interval_secs: u16) -> Self {
        let mut view = Self {
            state: ViewState::new(ViewOptions {
                preprocess: true,
                focusable: false,
                ..ViewOptions::default()
            }),
            interval_secs,
            last_update: Instant::now(),
            label_text: String::new(),
        };
        view.refresh_time();
        let w = view.label_text.len() as u16 + 2;
        view.state.set_bounds(Rect { x: 0, y: 0, w, h: 1 });
        view
    }

    fn refresh_time(&mut self) {
        let (h, m) = local_hm();
        self.label_text = format!("{h:02}:{m:02}");
        self.last_update = Instant::now();
        self.state.mark_dirty();
    }
}

impl View for ClockView {
    delegate_view_state!(state);

    fn draw(&mut self) {
        let style = Style {
            attrs: Attrs {
                reverse: true,
                ..Attrs::default()
            },
            ..Style::default()
        };
        let buf = self.state.buffer_mut();
        buf.fill(' ', style);
        buf.print(1, 0, &self.label_text, style);
        self.state.mark_redrawn();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Tick = event {
            if self.interval_secs > 0 && self.last_update.elapsed().as_secs() >= u64::from(self.interval_secs) {
                self.refresh_time();
            }
        }
        HandleResult::Ignored
    }
}

fn local_hm() -> (u32, u32) {
    use std::time::SystemTime;
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe { libc::localtime_r(&secs, &mut tm) };
    (tm.tm_hour as u32, tm.tm_min as u32)
}
