//! PrefixBinding — a single binding in the prefix map.

use txv_core::prelude::*;

/// A single binding in the prefix map.
pub(crate) struct PrefixBinding {
    pub(crate) key: char,
    pub(crate) command: CommandId,
    pub(crate) label: &'static str,
}
