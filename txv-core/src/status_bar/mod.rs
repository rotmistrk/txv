//! StatusBar module — Group with priority-based horizontal layout.

mod bar;
mod gravity;
mod layout;
mod status_slot;

pub use bar::StatusBar;
pub use gravity::Gravity;
pub use status_slot::StatusSlot;
