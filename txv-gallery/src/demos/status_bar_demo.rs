//! StatusBar demo widget for the gallery.

use txv_core::prelude::*;
use txv_core::status_bar::{Gravity, StatusBar, StatusSlot};
use txv_widgets::v_key_label::KeyLabelView;

/// Create a demo StatusBar with a few sample items.
pub(crate) fn make() -> Box<dyn View> {
    let mut bar = StatusBar::new();

    let kl1 = KeyLabelView::new(KeyEvent::new(KeyCode::F(1), KeyMod::NONE), 200, "F1 Help");
    bar.add(StatusSlot::new(Box::new(kl1)).priority(9));

    let kl2 = KeyLabelView::new(KeyEvent::new(KeyCode::F(10), KeyMod::NONE), 201, "F10 Quit");
    bar.add(StatusSlot::new(Box::new(kl2)).priority(9).gravity(Gravity::Right));

    Box::new(bar)
}
