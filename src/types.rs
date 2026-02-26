use std::path::PathBuf;

/// A structured match result.
///
/// ```rust
/// use ripgrep_api::SearchBuilder;
///
/// let mut matches = SearchBuilder::new("alpha")
///     .path(".")
///     .max_count(1)
///     .build()?
///     .collect::<Vec<_>>();
///
/// let first = matches.pop().unwrap();
/// println!("{}:{}", first.path.display(), first.line.unwrap_or(0));
/// # Ok::<(), ripgrep_api::SearchError>(())
/// ```
#[derive(Debug, Clone)]
pub struct Match {
    pub path: PathBuf,
    pub line: Option<u64>,
    pub column: Option<usize>,
    pub bytes: Vec<u8>,
    pub submatches: Vec<SubMatch>,
    pub line_text: String,
    pub context: Vec<ContextLine>,
}

#[derive(Debug, Clone)]
pub struct SubMatch {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct ContextLine {
    pub path: PathBuf,
    pub kind: ContextKind,
    pub line: Option<u64>,
    pub bytes: Vec<u8>,
    pub line_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextKind {
    Before,
    After,
    Other,
}
