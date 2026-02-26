use std::io::Read;
use std::path::{Path, PathBuf};

use crate::{
    config::CaseMode,
    config::Config,
    error::SearchError,
    search::Search,
    sink::MatchSink,
    types::{ContextLine, Match},
};

/// Fluent builder for rg-style search configuration.
///
/// ```rust
/// use ripgrep_api::SearchBuilder;
///
/// let matches: Vec<_> = SearchBuilder::new("todo")
///     .path(".")
///     .glob("**/*.rs")
///     .smart_case()
///     .build()?
///     .collect();
/// # Ok::<(), ripgrep_api::SearchError>(())
/// ```
pub struct SearchBuilder {
    config: Config,
}

impl SearchBuilder {
    pub fn new(pattern: impl Into<String>) -> Self {
        Self {
            config: Config::new(pattern.into()),
        }
    }

    pub fn path(mut self, path: impl AsRef<Path>) -> Self {
        self.config.paths = vec![path.as_ref().to_path_buf()];
        self
    }

    pub fn paths<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let collected: Vec<PathBuf> = paths
            .into_iter()
            .map(|p| p.as_ref().to_path_buf())
            .collect();
        if !collected.is_empty() {
            self.config.paths = collected;
        }
        self
    }

    pub fn build(self) -> Result<Search, SearchError> {
        Search::from_config(self.config)
    }

    pub fn search_with<S: MatchSink>(self, sink: &mut S) -> Result<(), SearchError> {
        crate::engine::search_with(&self.config, sink)
    }

    pub fn for_each<F>(self, on_match: F) -> Result<(), SearchError>
    where
        F: FnMut(&Match) -> bool,
    {
        struct MatchOnly<F>(F);

        impl<F> MatchSink for MatchOnly<F>
        where
            F: FnMut(&Match) -> bool,
        {
            fn matched(&mut self, mat: &Match) -> bool {
                (self.0)(mat)
            }
        }

        let mut sink = MatchOnly(on_match);
        crate::engine::search_with(&self.config, &mut sink)
    }

    pub fn for_each_with_context<F, C>(self, on_match: F, on_context: C) -> Result<(), SearchError>
    where
        F: FnMut(&Match) -> bool,
        C: FnMut(&ContextLine) -> bool,
    {
        struct MatchAndContext<F, C> {
            on_match: F,
            on_context: C,
        }

        impl<F, C> MatchSink for MatchAndContext<F, C>
        where
            F: FnMut(&Match) -> bool,
            C: FnMut(&ContextLine) -> bool,
        {
            fn matched(&mut self, mat: &Match) -> bool {
                (self.on_match)(mat)
            }

            fn context(&mut self, line: &ContextLine) -> bool {
                (self.on_context)(line)
            }
        }

        let mut sink = MatchAndContext {
            on_match,
            on_context,
        };
        crate::engine::search_with(&self.config, &mut sink)
    }

    pub fn search_reader<R: Read>(self, reader: R) -> Result<Vec<Match>, SearchError> {
        crate::engine::search_reader(&self.config, reader, Path::new("<reader>"))
    }

    pub fn search_reader_named<R, P>(self, source: P, reader: R) -> Result<Vec<Match>, SearchError>
    where
        R: Read,
        P: AsRef<Path>,
    {
        crate::engine::search_reader(&self.config, reader, source.as_ref())
    }

    pub fn search_slice(self, slice: &[u8]) -> Result<Vec<Match>, SearchError> {
        crate::engine::search_slice(&self.config, slice, Path::new("<memory>"))
    }

    pub fn search_slice_named<P>(self, source: P, slice: &[u8]) -> Result<Vec<Match>, SearchError>
    where
        P: AsRef<Path>,
    {
        crate::engine::search_slice(&self.config, slice, source.as_ref())
    }

    pub fn search_reader_with<R, S>(self, reader: R, sink: &mut S) -> Result<(), SearchError>
    where
        R: Read,
        S: MatchSink,
    {
        crate::engine::search_reader_with(&self.config, reader, Path::new("<reader>"), sink)
    }

    pub fn search_reader_with_named<R, P, S>(
        self,
        source: P,
        reader: R,
        sink: &mut S,
    ) -> Result<(), SearchError>
    where
        R: Read,
        P: AsRef<Path>,
        S: MatchSink,
    {
        crate::engine::search_reader_with(&self.config, reader, source.as_ref(), sink)
    }

    pub fn search_slice_with<S>(self, slice: &[u8], sink: &mut S) -> Result<(), SearchError>
    where
        S: MatchSink,
    {
        crate::engine::search_slice_with(&self.config, slice, Path::new("<memory>"), sink)
    }

    pub fn search_slice_with_named<P, S>(
        self,
        source: P,
        slice: &[u8],
        sink: &mut S,
    ) -> Result<(), SearchError>
    where
        P: AsRef<Path>,
        S: MatchSink,
    {
        crate::engine::search_slice_with(&self.config, slice, source.as_ref(), sink)
    }

    pub fn glob(mut self, pattern: impl Into<String>) -> Self {
        self.config.globs.push(pattern.into());
        self
    }

    pub fn type_(mut self, name: impl Into<String>) -> Self {
        self.config.types.push(name.into());
        self
    }

    pub fn type_not(mut self, name: impl Into<String>) -> Self {
        self.config.type_not.push(name.into());
        self
    }

    pub fn type_add(mut self, name: impl Into<String>, glob: impl Into<String>) -> Self {
        self.config.type_defs.push((name.into(), glob.into()));
        self
    }

    pub fn types(mut self, types: ignore::types::Types) -> Self {
        self.config.types_override = Some(types);
        self.config.types.clear();
        self.config.type_not.clear();
        self.config.type_defs.clear();
        self
    }

    pub fn overrides(mut self, overrides: ignore::overrides::Override) -> Self {
        self.config.overrides = Some(overrides);
        self.config.globs.clear();
        self
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.config.max_depth = Some(depth);
        self
    }

    pub fn max_filesize(mut self, bytes: u64) -> Self {
        self.config.max_filesize = Some(bytes);
        self
    }

    pub fn hidden(mut self) -> Self {
        self.config.search_hidden = true;
        self
    }

    pub fn follow(mut self) -> Self {
        self.config.follow_links = true;
        self
    }

    pub fn ignore(mut self, yes: bool) -> Self {
        self.config.ignore_files = yes;
        self.config.ignore_parent = yes;
        self.config.ignore_vcs = yes;
        self
    }

    pub fn ignore_parent(mut self, yes: bool) -> Self {
        self.config.ignore_parent = yes;
        self
    }

    pub fn ignore_files(mut self, yes: bool) -> Self {
        self.config.ignore_files = yes;
        self
    }

    pub fn ignore_vcs(mut self, yes: bool) -> Self {
        self.config.ignore_vcs = yes;
        self
    }

    pub fn smart_case(mut self) -> Self {
        self.config.case_mode = CaseMode::Smart;
        self
    }

    pub fn ignore_case(mut self) -> Self {
        self.config.case_mode = CaseMode::Insensitive;
        self
    }

    pub fn case_sensitive(mut self) -> Self {
        self.config.case_mode = CaseMode::Sensitive;
        self
    }

    pub fn fixed_strings(mut self) -> Self {
        self.config.fixed_strings = true;
        self
    }

    pub fn word(mut self) -> Self {
        self.config.word = true;
        self
    }

    pub fn line_regexp(mut self) -> Self {
        self.config.line_regexp = true;
        self
    }

    pub fn before_context(mut self, lines: usize) -> Self {
        self.config.before_context = lines;
        self
    }

    pub fn after_context(mut self, lines: usize) -> Self {
        self.config.after_context = lines;
        self
    }

    pub fn context(mut self, lines: usize) -> Self {
        self.config.before_context = lines;
        self.config.after_context = lines;
        self
    }

    pub fn max_count(mut self, count: usize) -> Self {
        self.config.max_count = Some(count);
        self
    }

    pub fn binary_detection(mut self, yes: bool) -> Self {
        self.config.binary_detection = yes;
        self
    }

    pub fn threads(mut self, threads: usize) -> Self {
        self.config.threads = Some(threads);
        self
    }

    pub fn memory_map(mut self, choice: grep_searcher::MmapChoice) -> Self {
        self.config.memory_map = Some(choice);
        self
    }

    pub fn heap_limit(mut self, bytes: usize) -> Self {
        self.config.heap_limit = Some(bytes);
        self
    }

    pub fn no_heap_limit(mut self) -> Self {
        self.config.heap_limit = None;
        self
    }

    pub fn engine_default(mut self) -> Self {
        self.config.engine = crate::config::RegexEngine::Default;
        self
    }

    #[cfg(feature = "pcre2")]
    pub fn pcre2(mut self) -> Self {
        self.config.engine = crate::config::RegexEngine::Pcre2;
        self
    }

    pub fn count(self) -> Result<u64, SearchError> {
        crate::engine::count(&self.config)
    }

    pub fn files_with_matches(self) -> Result<Vec<PathBuf>, SearchError> {
        crate::engine::files_with_matches(&self.config)
    }
}
