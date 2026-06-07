//! WakeFd — internal helper owning the pipe write end.

pub(super) struct WakeFd(pub(super) std::os::unix::io::RawFd);

impl Drop for WakeFd {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.0);
        }
    }
}

// SAFETY: the fd is only written to (single byte), which is atomic on pipes.
unsafe impl Send for WakeFd {}
unsafe impl Sync for WakeFd {}
