use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Match {
    pub path: PathBuf,
    pub line: Option<u64>,
    pub column: Option<usize>,
    pub bytes: Vec<u8>,
    pub submatches: Vec<SubMatch>,
    pub line_text: String,
}

#[derive(Debug, Clone)]
pub struct SubMatch {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct ContextLine {
    pub kind: ContextKind,
    pub line: Option<u64>,
    pub bytes: Vec<u8>,
    pub line_text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextKind {
    Before,
    After,
}
