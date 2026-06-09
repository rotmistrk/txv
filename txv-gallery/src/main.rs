//! txv-gallery — interactive widget gallery.
//!
//! Run with: cargo run -p txv-gallery

use txv_core::prelude::*;

fn main() {
    let color_mode = txv_render::detect_color_mode();
    let mut backend = txv_render::CrosstermBackend::new(color_mode);
    let (w, h) = backend.size();
    let mut app = txv_gallery::build_app();
    app.set_bounds(Rect::new(0, 0, w, h));
    run(&mut app, &mut backend);
}
