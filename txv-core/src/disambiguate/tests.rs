use super::*;

#[test]
fn unique_filenames_just_filename() {
    let names = &["src/main.rs", "src/lib.rs", "tests/helper.rs"];
    let result = disambiguate(names, '/', Side::Right);
    assert_eq!(result, vec!["main.rs", "lib.rs", "helper.rs"]);
}

#[test]
fn same_filename_different_dirs() {
    let names = &["some/path/package/lib/mod.rs", "other/path/project/core/lib/mod.rs"];
    let result = disambiguate(names, '/', Side::Right);
    // depth 1: "lib" vs "lib" — same! depth 2: "package" vs "core" — different!
    assert_eq!(result, vec!["package/…/mod.rs", "core/…/mod.rs"]);
}

#[test]
fn single_name_returns_filename() {
    let names = &["very/long/path/to/file.txt"];
    let result = disambiguate(names, '/', Side::Right);
    assert_eq!(result, vec!["file.txt"]);
}

#[test]
fn adjacent_segments_no_ellipsis() {
    let names = &["a/b/mod.rs", "a/c/mod.rs"];
    let result = disambiguate(names, '/', Side::Right);
    assert_eq!(result, vec!["b/mod.rs", "c/mod.rs"]);
}

#[test]
fn empty_input() {
    let result = disambiguate(&[], '/', Side::Right);
    assert!(result.is_empty());
}

#[test]
fn no_delimiter_in_names() {
    let names = &["alpha", "beta", "gamma"];
    let result = disambiguate(names, '/', Side::Right);
    assert_eq!(result, vec!["alpha", "beta", "gamma"]);
}

#[test]
fn identical_names_stay_identical() {
    let names = &["a/mod.rs", "b/mod.rs", "b/mod.rs"];
    let result = disambiguate(names, '/', Side::Right);
    // First two are distinguishable, last two are identical paths
    assert_eq!(result[0], "a/mod.rs");
    assert_eq!(result[1], "b/mod.rs");
    assert_eq!(result[2], "b/mod.rs");
}

#[test]
fn deep_path_with_gap() {
    let names = &[
        "workspace/alpha/src/utils/helpers/mod.rs",
        "workspace/beta/src/utils/helpers/mod.rs",
    ];
    let result = disambiguate(names, '/', Side::Right);
    // depth 1: helpers/helpers same, depth 2: utils/utils same,
    // depth 3: src/src same, depth 4: alpha/beta different!
    assert_eq!(result, vec!["alpha/…/mod.rs", "beta/…/mod.rs"]);
}

#[test]
fn left_side_preference() {
    let names = &["com.example.app", "org.example.app"];
    let result = disambiguate(names, '.', Side::Left);
    // anchor = "com" vs "org" — already unique
    assert_eq!(result, vec!["com", "org"]);
}

#[test]
fn three_way_needs_different_depths() {
    // Entry 0: package/lib/mod.rs — conflicts with 1,2 on "mod.rs"
    //   depth 1: "lib" — same as entry 1 ("lib"), different from entry 2 ("core")
    //   depth 2: "package" — different from entry 1 ("project") → depth 2
    // Entry 1: project/lib/mod.rs — depth 1: "lib" same as 0, diff from 2
    //   depth 2: "project" diff from 0 ("package") → depth 2
    // Entry 2: project/core/mod.rs — depth 1: "core" diff from 0,1 ("lib") → depth 1
    let names = &["package/lib/mod.rs", "project/lib/mod.rs", "project/core/mod.rs"];
    let result = disambiguate(names, '/', Side::Right);
    assert_eq!(result[0], "package/lib/mod.rs");
    assert_eq!(result[1], "project/lib/mod.rs");
    assert_eq!(result[2], "core/mod.rs");
}

#[test]
fn custom_ellipsis_glob() {
    let names = &[
        "workspace/alpha/src/utils/helpers/mod.rs",
        "workspace/beta/src/utils/helpers/mod.rs",
    ];
    let result = disambiguate_with(names, '/', Side::Right, "**");
    assert_eq!(result, vec!["alpha/**/mod.rs", "beta/**/mod.rs"]);
}
