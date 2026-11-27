// Marker for mcp-lint: #[cfg(test)]
//! Root labels survive rebuilds: two roots both named `web` keep their
//! disambiguated labels after a refresh (file-system change).

use std::fs;
use std::path::PathBuf;

use crate::tree_view::TreeData;

use super::FileTreeData;

fn two_web_roots() -> (tempfile::TempDir, Vec<PathBuf>) {
    let dir = tempfile::TempDir::new().unwrap_or_else(|e| panic!("tempdir: {e}"));
    let roots = vec![dir.path().join("app/web"), dir.path().join("lib/web")];
    for r in &roots {
        fs::create_dir_all(r.join("src")).unwrap_or_else(|e| panic!("mkdir: {e}"));
        fs::write(r.join("src/index.ts"), "export {};\n").unwrap_or_else(|e| panic!("write: {e}"));
    }
    (dir, roots)
}

fn root_labels(data: &FileTreeData) -> Vec<String> {
    (0..data.visible_count())
        .map(|row| data.visible_id(row))
        .filter(|&id| data.nodes[id].parent.is_none())
        .map(|id| data.label(id).to_string())
        .collect()
}

fn labelled() -> (tempfile::TempDir, FileTreeData) {
    let (dir, roots) = two_web_roots();
    let mut data = FileTreeData::with_roots(roots);
    data.set_root_labels(&["app/web".to_string(), "lib/web".to_string()]);
    (dir, data)
}

#[test]
fn labels_apply_to_root_nodes() {
    let (_dir, data) = labelled();
    assert_eq!(root_labels(&data), ["app/web", "lib/web"]);
}

#[test]
fn labels_survive_refresh() {
    let (_dir, mut data) = labelled();
    data.refresh();
    data.rebuild_visible();
    assert_eq!(root_labels(&data), ["app/web", "lib/web"]);
}

#[test]
fn new_roots_drop_old_labels() {
    let (dir, mut data) = labelled();
    let other = dir.path().join("tools");
    fs::create_dir_all(&other).unwrap_or_else(|e| panic!("mkdir: {e}"));
    data.set_roots(vec![dir.path().join("app/web"), other]);
    data.rebuild_visible();
    assert_eq!(root_labels(&data), ["web", "tools"]);
}

#[test]
fn same_relative_path_in_two_roots_keeps_its_own_color() {
    use std::collections::HashMap;
    use txv_core::cell::Color;

    /// Any distinct color marks the file.
    const MARK: Color = Color::Ansi(1);

    let (dir, roots) = two_web_roots();
    let mut data = FileTreeData::with_roots(roots.clone());
    data.expand_paths(&[roots[0].join("src"), roots[1].join("src")]);
    let app_index = roots[0].join("src/index.ts");
    let mut colors = HashMap::new();
    colors.insert(app_index.clone(), MARK);
    data.set_colors(colors);
    let style_of = |data: &FileTreeData, path: &std::path::Path| {
        let id = data
            .nodes
            .iter()
            .position(|n| n.path == path)
            .unwrap_or_else(|| panic!("node {path:?}"));
        data.style(id).fg()
    };
    assert_eq!(style_of(&data, &app_index), MARK);
    assert_ne!(style_of(&data, &roots[1].join("src/index.ts")), MARK);
    drop(dir);
}
