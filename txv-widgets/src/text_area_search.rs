//! TextArea — search methods.

use crate::text_area::TextArea;

impl TextArea {
    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn search_matches(&self) -> &[usize] {
        &self.search_matches
    }

    pub fn current_match(&self) -> usize {
        self.current_match
    }

    pub fn search(&mut self, query: &str) {
        self.search_query = query.to_string();
        self.search_matches.clear();
        if !query.is_empty() {
            for (i, line) in self.lines.iter().enumerate() {
                if line.contains(query) {
                    self.search_matches.push(i);
                }
            }
        }
        self.current_match = 0;
        if let Some(&line) = self.search_matches.first() {
            self.scroll.ensure_visible(line);
        }
        self.state.mark_dirty();
    }

    pub fn next_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = (self.current_match + 1) % self.search_matches.len();
        let line = self.search_matches[self.current_match];
        self.scroll.ensure_visible(line);
        self.state.mark_dirty();
    }

    pub fn prev_match(&mut self) {
        if self.search_matches.is_empty() {
            return;
        }
        self.current_match = if self.current_match == 0 {
            self.search_matches.len() - 1
        } else {
            self.current_match - 1
        };
        let line = self.search_matches[self.current_match];
        self.scroll.ensure_visible(line);
        self.state.mark_dirty();
    }
}
