//! txv-gallery — widget gallery app for demonstration and testing.
//!
//! Layout: left=widget list, center=demo widget, right=source code.
//! The `build_app` function constructs the root view.

mod app;
mod demos;
mod snippets;
mod widget_list;

pub use app::build_app;
