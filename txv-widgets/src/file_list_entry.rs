//! FileEntry — a single entry in a flat file list.

use std::path::PathBuf;

pub(crate) struct FileEntry {
    pub(crate) path: PathBuf,
    pub(crate) display: String,
}
