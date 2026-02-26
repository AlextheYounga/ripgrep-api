use std::path::{Path, PathBuf};

use crate::{config::Config, error::SearchError, search::Search};

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
}
