use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use ripgrep_api::{ContextKind, MatchSink, SearchBuilder};

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

fn rel(path: &Path, root: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap().to_path_buf()
}

#[test]
fn default_search_respects_ignore_and_hidden() {
    let root = fixture_root();
    let results: Vec<_> = SearchBuilder::new("alpha")
        .path(&root)
        .build()
        .unwrap()
        .collect();

    let files: BTreeSet<_> = results.iter().map(|m| rel(&m.path, &root)).collect();

    assert!(!files.contains(Path::new("ignored.txt")));
    assert!(!files.contains(Path::new(".hidden.txt")));
    assert!(files.contains(Path::new("root.txt")));
    assert!(files.contains(Path::new("nested/inner.rs")));
}

#[test]
fn hidden_files_can_be_included() {
    let root = fixture_root();
    let results: Vec<_> = SearchBuilder::new("alpha")
        .path(&root)
        .hidden()
        .build()
        .unwrap()
        .collect();

    let files: BTreeSet<_> = results.iter().map(|m| rel(&m.path, &root)).collect();

    assert!(files.contains(Path::new(".hidden.txt")));
}

#[test]
fn glob_filters_results() {
    let root = fixture_root();
    let results: Vec<_> = SearchBuilder::new("alpha")
        .path(&root)
        .glob("**/*.rs")
        .build()
        .unwrap()
        .collect();

    let files: BTreeSet<_> = results.iter().map(|m| rel(&m.path, &root)).collect();

    assert_eq!(files, BTreeSet::from([PathBuf::from("nested/inner.rs")]));
}

#[test]
fn type_filters_results() {
    let root = fixture_root();
    let results: Vec<_> = SearchBuilder::new("alpha")
        .path(&root)
        .type_("rust")
        .build()
        .unwrap()
        .collect();

    let files: BTreeSet<_> = results.iter().map(|m| rel(&m.path, &root)).collect();

    assert_eq!(files, BTreeSet::from([PathBuf::from("nested/inner.rs")]));
}

#[test]
fn max_depth_limits_walk() {
    let root = fixture_root();
    let results: Vec<_> = SearchBuilder::new("alpha")
        .path(&root)
        .max_depth(1)
        .build()
        .unwrap()
        .collect();

    let files: BTreeSet<_> = results.iter().map(|m| rel(&m.path, &root)).collect();

    assert!(!files.contains(Path::new("nested/deeper/deep.txt")));
}

#[test]
fn max_filesize_skips_large_files() {
    let root = fixture_root();
    let results: Vec<_> = SearchBuilder::new("x")
        .path(&root)
        .max_filesize(10)
        .build()
        .unwrap()
        .collect();

    assert!(results.is_empty());
}

#[test]
fn smart_case_defaults_apply() {
    let root = fixture_root();
    let results: Vec<_> = SearchBuilder::new("Alpha")
        .path(&root)
        .build()
        .unwrap()
        .collect();

    assert_eq!(results.len(), 1);
    assert!(results[0].line_text.contains("Alpha"));
}

#[test]
fn context_lines_are_collected() {
    let root = fixture_root();
    let results: Vec<_> = SearchBuilder::new("match")
        .path(root.join("context.txt"))
        .context(1)
        .build()
        .unwrap()
        .collect();

    assert_eq!(results.len(), 1);
    let context = &results[0].context;
    assert_eq!(context.len(), 2);

    let kinds: BTreeSet<_> = context.iter().map(|line| line.kind).collect();
    assert!(kinds.contains(&ContextKind::Before));
    assert!(kinds.contains(&ContextKind::After));
}

#[test]
fn max_count_caps_matches_per_file() {
    let root = fixture_root();
    let results: Vec<_> = SearchBuilder::new("alpha")
        .path(&root)
        .max_count(1)
        .build()
        .unwrap()
        .collect();

    let root_matches: Vec<_> = results
        .iter()
        .filter(|m| rel(&m.path, &root) == PathBuf::from("root.txt"))
        .collect();
    assert_eq!(root_matches.len(), 1);
}

#[test]
fn count_returns_total_matches() {
    let root = fixture_root();
    let total = SearchBuilder::new("alpha").path(&root).count().unwrap();
    assert_eq!(total, 5);
}

#[test]
fn files_with_matches_returns_unique_paths() {
    let root = fixture_root();
    let files = SearchBuilder::new("alpha")
        .path(&root)
        .files_with_matches()
        .unwrap();

    let rel_files: BTreeSet<_> = files.iter().map(|path| rel(path, &root)).collect();

    assert_eq!(
        rel_files,
        BTreeSet::from([
            PathBuf::from("custom.foo"),
            PathBuf::from("nested/deeper/deep.txt"),
            PathBuf::from("nested/inner.rs"),
            PathBuf::from("root.txt"),
        ])
    );
}

#[test]
fn type_definitions_can_be_added() {
    let root = fixture_root();
    let results: Vec<_> = SearchBuilder::new("alpha")
        .path(&root)
        .type_add("custom", "*.foo")
        .type_("custom")
        .build()
        .unwrap()
        .collect();

    let files: BTreeSet<_> = results.iter().map(|m| rel(&m.path, &root)).collect();

    assert_eq!(files, BTreeSet::from([PathBuf::from("custom.foo")]));
}

#[test]
fn overrides_can_whitelist_ignored_files() {
    let root = fixture_root();
    let mut builder = ignore::overrides::OverrideBuilder::new(&root);
    builder.add("ignored.txt").unwrap();
    let overrides = builder.build().unwrap();

    let results: Vec<_> = SearchBuilder::new("alpha")
        .path(&root)
        .overrides(overrides)
        .build()
        .unwrap()
        .collect();

    let files: BTreeSet<_> = results.iter().map(|m| rel(&m.path, &root)).collect();

    assert!(files.contains(Path::new("ignored.txt")));
}

#[test]
fn search_slice_works() {
    let haystack = b"zero\nmatch\nthree\n";
    let results: Vec<_> = SearchBuilder::new("match").search_slice(haystack).unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].line_text.contains("match"));
}

#[test]
fn search_reader_works() {
    let data = b"alpha\n".to_vec();
    let results: Vec<_> = SearchBuilder::new("alpha")
        .search_reader(std::io::Cursor::new(data))
        .unwrap();

    assert_eq!(results.len(), 1);
}

#[test]
fn streaming_callbacks_receive_matches_and_context() {
    struct Counter {
        matches: usize,
        contexts: usize,
    }

    impl MatchSink for Counter {
        fn matched(&mut self, _mat: &ripgrep_api::Match) -> bool {
            self.matches += 1;
            true
        }

        fn context(&mut self, _line: &ripgrep_api::ContextLine) -> bool {
            self.contexts += 1;
            true
        }
    }

    let root = fixture_root();
    let mut counter = Counter {
        matches: 0,
        contexts: 0,
    };

    SearchBuilder::new("match")
        .path(root.join("context.txt"))
        .context(1)
        .search_with(&mut counter)
        .unwrap();

    assert_eq!(counter.matches, 1);
    assert_eq!(counter.contexts, 2);
}

#[test]
fn walk_files_respects_globs_and_types() {
    let root = fixture_root();
    let files = SearchBuilder::new("irrelevant")
        .path(&root)
        .glob("**/*.rs")
        .walk_files()
        .unwrap();

    let rel_files: BTreeSet<_> = files.iter().map(|path| rel(path, &root)).collect();

    assert_eq!(
        rel_files,
        BTreeSet::from([PathBuf::from("nested/inner.rs")])
    );

    let files = SearchBuilder::new("irrelevant")
        .path(&root)
        .type_("rust")
        .walk_files()
        .unwrap();

    let rel_files: BTreeSet<_> = files.iter().map(|path| rel(path, &root)).collect();

    assert_eq!(
        rel_files,
        BTreeSet::from([PathBuf::from("nested/inner.rs")])
    );
}

#[test]
fn walk_files_respects_ignore_and_hidden() {
    let root = fixture_root();
    let files = SearchBuilder::new("irrelevant")
        .path(&root)
        .walk_files()
        .unwrap();

    let rel_files: BTreeSet<_> = files.iter().map(|path| rel(path, &root)).collect();

    assert!(!rel_files.contains(Path::new("ignored.txt")));
    assert!(!rel_files.contains(Path::new(".hidden.txt")));

    let files = SearchBuilder::new("irrelevant")
        .path(&root)
        .hidden()
        .walk_files()
        .unwrap();

    let rel_files: BTreeSet<_> = files.iter().map(|path| rel(path, &root)).collect();

    assert!(rel_files.contains(Path::new(".hidden.txt")));
}

#[test]
fn limit_caps_total_results() {
    let root = fixture_root();
    let results: Vec<_> = SearchBuilder::new("alpha")
        .path(&root)
        .limit(2)
        .build()
        .unwrap()
        .collect();

    assert_eq!(results.len(), 2);
}

#[test]
fn limit_caps_count() {
    let root = fixture_root();
    let total = SearchBuilder::new("alpha")
        .path(&root)
        .limit(3)
        .count()
        .unwrap();

    assert_eq!(total, 3);
}

#[test]
fn limit_above_total_returns_all() {
    let root = fixture_root();
    let results: Vec<_> = SearchBuilder::new("alpha")
        .path(&root)
        .limit(100)
        .build()
        .unwrap()
        .collect();

    assert_eq!(results.len(), 5);
}

#[test]
fn limit_works_with_for_each() {
    let root = fixture_root();
    let mut count = 0usize;
    SearchBuilder::new("alpha")
        .path(&root)
        .limit(2)
        .for_each(|_mat| {
            count += 1;
            true
        })
        .unwrap();

    assert_eq!(count, 2);
}

#[test]
fn limit_works_with_search_slice() {
    let haystack = b"alpha\nalpha\nalpha\nalpha\n";
    let results = SearchBuilder::new("alpha")
        .limit(2)
        .search_slice(haystack)
        .unwrap();

    assert_eq!(results.len(), 2);
}

#[test]
fn limit_combined_with_max_count() {
    // max_count=1 means at most 1 match per file; limit=2 means at most 2 total.
    // There are 4 files containing "alpha", so max_count alone would give 4.
    let root = fixture_root();
    let results: Vec<_> = SearchBuilder::new("alpha")
        .path(&root)
        .max_count(1)
        .limit(2)
        .build()
        .unwrap()
        .collect();

    assert_eq!(results.len(), 2);
}

#[test]
fn walk_files_returns_only_files() {
    let root = fixture_root();
    let files = SearchBuilder::new("irrelevant")
        .path(&root)
        .walk_files()
        .unwrap();

    let rel_files: BTreeSet<_> = files.iter().map(|path| rel(path, &root)).collect();

    assert!(!rel_files.contains(Path::new("nested")));
    assert!(!rel_files.contains(Path::new("nested/deeper")));
    assert!(rel_files.contains(Path::new("nested/deeper/deep.txt")));
}
