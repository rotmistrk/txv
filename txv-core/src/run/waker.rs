//! Waker — handle that wakes the event loop from any thread.

use std::os::unix::io::RawFd;
use std::sync::Arc;

use super::wake_fd::WakeFd;

/// A handle that wakes the event loop from any thread.
/// Clone + Send — safe to pass to background threads.
#[derive(Clone)]
pub struct Waker {
    fd: Option<Arc<WakeFd>>,
}

impl Waker {
    /// Create a waker from the write end of a pipe.
    pub fn from_fd(write_fd: RawFd) -> Self {
        Self {
            fd: Some(Arc::new(WakeFd(write_fd))),
        }
    }

    /// No-op waker (for tests/mock backends).
    pub fn noop() -> Self {
        Self { fd: None }
    }

    /// Wake the event loop. Safe to call from any thread.
    pub fn wake(&self) {
        if let Some(fd) = &self.fd {
            unsafe {
                libc::write(fd.0, b"W".as_ptr() as *const libc::c_void, 1);
            }
        }
    }
}
