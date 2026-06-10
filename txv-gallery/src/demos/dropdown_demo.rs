//! DropdownMenu demo for the gallery.

use txv_core::prelude::*;
use txv_widgets::dropdown_menu::DropdownMenu;
use txv_widgets::dropdown_source::DropdownSource;

/// Demo source: programming languages with categories.
pub(crate) struct LangSource {
    items: Vec<(&'static str, &'static str)>,
    visible: Vec<usize>,
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
        let visible = (0..items.len()).collect();
        Self { items, visible }
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
    fn filter(&mut self, query: &str) {
        self.visible = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, (name, _))| name.to_lowercase().contains(&query.to_lowercase()))
            .map(|(i, _)| i)
            .collect();
    }
    fn visible_len(&self) -> usize {
        self.visible.len()
    }
    fn visible_index(&self, visible_idx: usize) -> usize {
        self.visible[visible_idx]
    }
}

pub(crate) fn make() -> Box<dyn View> {
    use txv_widgets::dropdown_menu::NumberMode;
    let source = LangSource::new();
    let dd = DropdownMenu::new(source).with_numbers(NumberMode::All);
    Box::new(dd)
}
