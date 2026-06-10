//! DropdownMenu demo for the gallery.

use txv_core::prelude::*;
use txv_widgets::dropdown_menu::{DropdownMenu, NumberMode};
use txv_widgets::dropdown_source::DropdownSource;

/// Demo source: programming languages with categories.
pub(crate) struct LangSource {
    items: Vec<(&'static str, &'static str)>,
}

impl LangSource {
    pub(crate) fn new() -> Self {
        let items = vec![
            ("Rust", "systems"),
            ("Go", "systems"),
            ("Python", "scripting"),
            ("TypeScript", "scripting"),
            ("Haskell", "functional"),
            ("OCaml", "functional"),
            ("C++", "systems"),
            ("Zig", "systems"),
            ("Lua", "scripting"),
            ("Elixir", "functional"),
        ];
        Self { items }
    }
}

impl DropdownSource for LangSource {
    fn len(&self) -> usize {
        self.items.len()
    }
    fn label(&self, idx: usize) -> &str {
        self.items[idx].0
    }
    fn secondary(&self, idx: usize) -> &str {
        self.items[idx].1
    }
    fn badge(&self, idx: usize) -> Option<(char, Style)> {
        let (ch, color) = match self.items[idx].1 {
            "systems" => ('●', Color::Ansi(1)),
            "functional" => ('λ', Color::Ansi(5)),
            "scripting" => ('⚡', Color::Ansi(3)),
            _ => return None,
        };
        Some((ch, Style::default().with_fg(color)))
    }
}

pub(crate) fn make() -> Box<dyn View> {
    let source = LangSource::new();
    let dd = DropdownMenu::new(source).with_numbers(NumberMode::All);
    Box::new(dd)
}
