use super::*;
use crate::file_tree::FileTreeData;
use crate::tree_view::TreeData;

#[test]
fn fuzzy_match_basic() {
    let result = fuzzy_match_positions("movement.rs", "mvt");
    assert_eq!(result, Some(vec![0, 2, 7]));
}

#[test]
fn fuzzy_match_no_match() {
    let result = fuzzy_match_positions("hello", "xyz");
    assert_eq!(result, None);
}

#[test]
fn fuzzy_match_exact() {
    let result = fuzzy_match_positions("mod.rs", "mod");
    assert_eq!(result, Some(vec![0, 1, 2]));
}

#[test]
fn filter_hides_non_matching_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("main.rs"), "").expect("write");
    std::fs::write(dir.path().join("lib.rs"), "").expect("write");
    std::fs::write(dir.path().join("test.txt"), "").expect("write");

    let mut data = FileTreeData::new(dir.path());
    data.set_filter("rs");
    let visible: Vec<&str> = (0..data.visible_count())
        .map(|i| data.label(data.visible_id(i)))
        .collect();
    assert!(!visible.contains(&"test.txt"));
    assert!(visible.contains(&"main.rs"));
    assert!(visible.contains(&"lib.rs"));
}

#[test]
fn clear_filter_restores_all() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("main.rs"), "").expect("write");
    std::fs::write(dir.path().join("test.txt"), "").expect("write");

    let mut data = FileTreeData::new(dir.path());
    let total = data.visible_count();
    data.set_filter("rs");
    assert!(data.visible_count() < total);
    data.set_filter("");
    assert_eq!(data.visible_count(), total);
}

#[test]
fn filter_match_positions_recorded() {
    let dir = tempfile::tempdir().expect("tempdir");
    std::fs::write(dir.path().join("movement.rs"), "").expect("write");

    let mut data = FileTreeData::new(dir.path());
    data.set_filter("mvt");
    let mut found = false;
    for i in 0..data.visible_count() {
        let id = data.visible_id(i);
        if data.label(id) == "movement.rs" {
            assert_eq!(data.match_positions(id), Some([0, 2, 7].as_slice()));
            found = true;
        }
    }
    assert!(found);
}

#[test]
fn filter_shows_closed_dir_with_matches_inside() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sub = dir.path().join("src");
    std::fs::create_dir(&sub).expect("mkdir");
    std::fs::write(sub.join("deep.rs"), "").expect("write");
    std::fs::write(dir.path().join("top.txt"), "").expect("write");

    let mut data = FileTreeData::new(dir.path());
    data.ensure_all_loaded();
    data.set_filter("deep");
    let vis = |d: &FileTreeData| -> Vec<String> {
        (0..d.visible_count())
            .map(|i| d.label(d.visible_id(i)).to_string())
            .collect()
    };
    assert!(vis(&data).contains(&"src".to_string()));
    assert!(!vis(&data).contains(&"deep.rs".to_string()));
    let src_id = data.nodes.iter().position(|n| n.label == "src").expect("src");
    data.toggle(src_id);
    assert!(vis(&data).contains(&"deep.rs".to_string()));
}

#[test]
fn filter_shows_children_of_expanded_dir() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sub = dir.path().join("src");
    std::fs::create_dir(&sub).expect("mkdir");
    std::fs::write(sub.join("deep.rs"), "").expect("write");
    std::fs::write(sub.join("other.txt"), "").expect("write");

    let mut data = FileTreeData::new(dir.path());
    data.ensure_all_loaded();
    let src_id = data.nodes.iter().position(|n| n.label == "src").expect("src");
    data.toggle(src_id);
    data.set_filter("deep");
    let visible: Vec<&str> = (0..data.visible_count())
        .map(|i| data.label(data.visible_id(i)))
        .collect();
    assert!(visible.contains(&"src") && visible.contains(&"deep.rs"));
    assert!(!visible.contains(&"other.txt"));
}

#[test]
fn dir_name_match_shows_all_children() {
    let dir = tempfile::tempdir().expect("tempdir");
    let hooks = dir.path().join("hooks");
    std::fs::create_dir(&hooks).expect("mkdir");
    std::fs::write(hooks.join("pre-commit"), "").expect("write");
    std::fs::write(hooks.join("post-merge"), "").expect("write");

    let mut data = FileTreeData::new(dir.path());
    data.ensure_all_loaded();
    let hooks_id = data.nodes.iter().position(|n| n.label == "hooks").expect("hooks");
    data.toggle(hooks_id);
    data.set_filter("hooks");
    let visible: Vec<&str> = (0..data.visible_count())
        .map(|i| data.label(data.visible_id(i)))
        .collect();
    assert!(visible.contains(&"hooks") && visible.contains(&"pre-commit"));
    assert!(visible.contains(&"post-merge"));
}

#[test]
fn collapse_during_filter_hides_children_keeps_dir() {
    let dir = tempfile::tempdir().expect("tempdir");
    let doc = dir.path().join("doc");
    std::fs::create_dir(&doc).expect("mkdir");
    std::fs::write(doc.join("readme.md"), "").expect("write");

    let mut data = FileTreeData::new(dir.path());
    data.ensure_all_loaded();
    let doc_id = data.nodes.iter().position(|n| n.label == "doc").expect("doc");
    data.toggle(doc_id);
    data.set_filter("md");
    let vis = |d: &FileTreeData| -> Vec<String> {
        (0..d.visible_count())
            .map(|i| d.label(d.visible_id(i)).to_string())
            .collect()
    };
    assert!(vis(&data).contains(&"readme.md".to_string()));
    data.toggle(doc_id);
    assert!(vis(&data).contains(&"doc".to_string()), "dir stays visible");
    assert!(!vis(&data).contains(&"readme.md".to_string()), "children hidden");
}
