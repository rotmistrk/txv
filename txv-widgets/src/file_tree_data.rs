//! TreeData trait implementation for FileTreeData.

use txv_core::cell::{Color, Style};
use txv_core::palette::{palette, StyleId};

use crate::file_tree::FileTreeData;
use crate::tree_view::TreeData;

impl TreeData for FileTreeData {
    fn root_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.parent.is_none()).count()
    }

    fn child_count(&self, id: usize) -> usize {
        self.nodes.iter().filter(|n| n.parent == Some(id)).count()
    }

    fn label(&self, id: usize) -> &str {
        &self.nodes[id].label
    }

    fn is_expandable(&self, id: usize) -> bool {
        self.nodes[id].is_dir
    }

    fn is_expanded(&self, id: usize) -> bool {
        self.nodes[id].expanded
    }

    fn toggle(&mut self, id: usize) {
        if self.nodes[id].expanded {
            self.collapse_node(id);
        } else {
            self.expand_node(id);
        }
    }

    fn depth(&self, id: usize) -> usize {
        self.nodes[id].depth
    }

    fn visible_count(&self) -> usize {
        self.visible.len()
    }

    fn visible_id(&self, row: usize) -> usize {
        self.visible[row]
    }

    fn style(&self, id: usize) -> Style {
        let node = &self.nodes[id];
        if node.ignored {
            return palette().style(StyleId::Dim);
        }
        if node.is_dir {
            return palette().style(StyleId::TreeDir);
        }
        let root = self.root_of(id);
        let rel = node.path.strip_prefix(root).ok().and_then(|p| p.to_str());
        if let Some(rel_path) = rel {
            if let Some(&color) = self.colors.get(rel_path) {
                return Style {
                    fg: color,
                    ..Style::default()
                };
            }
        }
        Style::default()
    }

    fn highlight_positions(&self, id: usize) -> Option<&[usize]> {
        self.match_positions.get(&id).map(|v| v.as_slice())
    }

    fn filter_status(&self) -> Option<&str> {
        if self.filter.is_empty() {
            None
        } else {
            Some(&self.filter)
        }
    }

    fn badge_color(&self, id: usize) -> Option<Color> {
        if !self.is_multi_root() || self.root_badge_colors.is_empty() {
            return None;
        }
        let node = &self.nodes[id];
        if node.depth != 0 || node.parent.is_some() {
            return None;
        }
        // Find root index by matching path
        let idx = self.extra_roots.iter().position(|r| r == &node.path)?;
        self.root_badge_colors.get(idx).copied()
    }

    fn is_open(&self, id: usize) -> bool {
        let node = &self.nodes[id];
        !node.is_dir && self.open_files.contains(&node.path)
    }

    fn icon(&self, id: usize) -> Option<&str> {
        if !self.show_icons {
            return None;
        }
        let node = &self.nodes[id];
        if node.is_dir {
            Some("📁")
        } else {
            Some(icon_for_extension(&node.label))
        }
    }
}

fn icon_for_extension(filename: &str) -> &'static str {
    let ext = filename.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => "🦀",
        "py" => "🐍",
        "js" | "mjs" | "cjs" => "JS",
        "ts" | "mts" => "TS",
        "tsx" | "jsx" => "⚛ ",
        "go" => "Go",
        "java" => "☕",
        "c" | "h" => "C ",
        "cpp" | "cxx" | "cc" | "hpp" => "++",
        "rb" => "💎",
        "lua" => "🌙",
        "sh" | "bash" | "zsh" => "$ ",
        "json" | "jsonc" | "jsonl" => "{}",
        "toml" | "ini" | "cfg" => "⚙ ",
        "yaml" | "yml" => "📋",
        "xml" => "◇ ",
        "html" | "htm" => "🌐",
        "css" | "scss" | "less" => "🎨",
        "md" | "markdown" => "📝",
        "txt" | "text" => "📄",
        "lock" => "🔒",
        "gitignore" | "gitmodules" | "gitattributes" => "⎇ ",
        "dockerfile" => "🐳",
        "svg" | "png" | "jpg" | "jpeg" | "gif" | "ico" => "🖼 ",
        "sql" => "🗃 ",
        "log" => "📜",
        _ if filename == "Makefile" || filename == "CMakeLists.txt" => "🔧",
        _ if filename == "Cargo.toml" => "📦",
        _ => "· ",
    }
}
