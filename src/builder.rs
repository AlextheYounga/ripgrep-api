use std::path::{Path, PathBuf};

use crate::{config::CaseMode, config::Config, error::SearchError, search::Search};

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

    pub fn count(self) -> Result<u64, SearchError> {
        crate::engine::count(&self.config)
    }

    pub fn files_with_matches(self) -> Result<Vec<PathBuf>, SearchError> {
        crate::engine::files_with_matches(&self.config)
    }
}
