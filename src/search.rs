use crate::{config::Config, engine, error::SearchError, types::Match};

pub struct Search {
    inner: std::vec::IntoIter<Match>,
}

impl Search {
    pub(crate) fn from_config(config: Config) -> Result<Self, SearchError> {
        let results = engine::search(&config)?;
        Ok(Self {
            inner: results.into_iter(),
        })
    }
}

impl Iterator for Search {
    type Item = Match;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}
