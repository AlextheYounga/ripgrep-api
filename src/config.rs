use std::path::PathBuf;

use ignore::{overrides::Override, types::Types};

#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub(crate) pattern: String,
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) globs: Vec<String>,
    pub(crate) types: Vec<String>,
    pub(crate) type_not: Vec<String>,
    pub(crate) type_defs: Vec<(String, String)>,
    pub(crate) overrides: Option<Override>,
    pub(crate) types_override: Option<Types>,
    pub(crate) max_depth: Option<usize>,
    pub(crate) max_filesize: Option<u64>,
    pub(crate) search_hidden: bool,
    pub(crate) follow_links: bool,
    pub(crate) ignore_files: bool,
    pub(crate) ignore_parent: bool,
    pub(crate) ignore_vcs: bool,
    pub(crate) before_context: usize,
    pub(crate) after_context: usize,
    pub(crate) max_count: Option<usize>,
    pub(crate) case_mode: CaseMode,
    pub(crate) fixed_strings: bool,
    pub(crate) word: bool,
    pub(crate) line_regexp: bool,
    pub(crate) binary_detection: bool,
    pub(crate) engine: RegexEngine,
    pub(crate) threads: Option<usize>,
    pub(crate) memory_map: Option<grep_searcher::MmapChoice>,
    pub(crate) heap_limit: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CaseMode {
    Smart,
    Insensitive,
    Sensitive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RegexEngine {
    Default,
    #[cfg(feature = "pcre2")]
    Pcre2,
}

impl Config {
    pub(crate) fn new(pattern: String) -> Self {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            pattern,
            paths: vec![root],
            globs: Vec::new(),
            types: Vec::new(),
            type_not: Vec::new(),
            type_defs: Vec::new(),
            overrides: None,
            types_override: None,
            max_depth: None,
            max_filesize: None,
            search_hidden: false,
            follow_links: false,
            ignore_files: true,
            ignore_parent: true,
            ignore_vcs: true,
            before_context: 0,
            after_context: 0,
            max_count: None,
            case_mode: CaseMode::Smart,
            fixed_strings: false,
            word: false,
            line_regexp: false,
            binary_detection: true,
            engine: RegexEngine::Default,
            threads: None,
            memory_map: None,
            heap_limit: None,
        }
    }
}
