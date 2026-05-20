//! Style attribute emission helpers for crossterm backend.

use std::io::Write;

use crossterm::{
    queue,
    style::{Attribute, SetAttribute},
};
use txv_core::cell::Attrs;

pub(crate) fn emit_attrs(out: &mut impl Write, attrs: Attrs) {
    if attrs.bold {
        queue!(out, SetAttribute(Attribute::Bold)).ok();
    }
    if attrs.dim {
        queue!(out, SetAttribute(Attribute::Dim)).ok();
    }
    if attrs.italic {
        queue!(out, SetAttribute(Attribute::Italic)).ok();
    }
    if attrs.underline {
        queue!(out, SetAttribute(Attribute::Underlined)).ok();
    }
    if attrs.reverse {
        queue!(out, SetAttribute(Attribute::Reverse)).ok();
    }
}
