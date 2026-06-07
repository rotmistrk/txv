//! ClockItem — displays current time.

use std::mem;
use std::time::Instant;

use txv_core::status::{Gravity, VisibleItem};

pub struct ClockItem {
    interval: u16,
    last_update: Instant,
    label_text: String,
    gravity: Gravity,
}

impl ClockItem {
    pub fn new(interval: u16) -> Self {
        let mut item = Self {
            interval,
            last_update: Instant::now(),
            label_text: String::new(),
            gravity: Gravity::Right,
        };
        item.refresh_time();
        item
    }
    pub fn with_gravity(mut self, g: Gravity) -> Self {
        self.gravity = g;
        self
    }

    fn refresh_time(&mut self) {
        let (h, m) = local_hm();
        self.label_text = format!("{h:02}:{m:02}");
        self.last_update = Instant::now();
    }
}

impl VisibleItem for ClockItem {
    fn label(&self) -> &str {
        &self.label_text
    }
    fn gravity(&self) -> Gravity {
        self.gravity
    }
    fn tick(&mut self) {
        if self.interval > 0 && self.last_update.elapsed().as_secs() >= u64::from(self.interval) {
            self.refresh_time();
        }
    }
}

fn local_hm() -> (u32, u32) {
    use std::time::SystemTime;
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as libc::time_t;
    let mut tm: libc::tm = unsafe { mem::zeroed() };
    unsafe { libc::localtime_r(&secs, &mut tm) };
    (tm.tm_hour as u32, tm.tm_min as u32)
}
