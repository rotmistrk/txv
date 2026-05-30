//! Tests for InputLine selection behavior.

#[cfg(test)]
mod tests {
    use txv_core::event::{KeyCode, KeyEvent, KeyMod};
    use txv_core::prelude::*;

    use crate::InputLine;

    fn key(code: KeyCode) -> Event {
        Event::Key(KeyEvent {
            code,
            modifiers: KeyMod {
                ctrl: false,
                alt: false,
                shift: false,
            },
        })
    }

    #[test]
    fn right_at_end_clears_selection() {
        let mut input = InputLine::new().with_command(100);
        input.set_text("hello");
        input.select_all();
        assert!(input.selection_range().is_some());
        input.handle(&key(KeyCode::Right));
        assert!(input.selection_range().is_none());
    }

    #[test]
    fn left_at_start_clears_selection() {
        let mut input = InputLine::new().with_command(100);
        input.set_text("hello");
        // select_all: cursor=5, anchor=0
        input.select_all();
        // Left: cursor moves to 4, clears selection
        input.handle(&key(KeyCode::Left));
        assert!(input.selection_range().is_none());
    }

    #[test]
    fn right_mid_text_clears_selection_and_moves() {
        let mut input = InputLine::new().with_command(100);
        input.set_text("hello");
        input.handle_nav(false, 2);
        input.handle_nav(true, 4); // shift-nav: select 2..4
        assert!(input.selection_range().is_some());
        input.handle(&key(KeyCode::Right));
        assert!(input.selection_range().is_none());
    }
}
