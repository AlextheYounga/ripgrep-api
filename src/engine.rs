use std::io;
use std::path::Path;

use grep_matcher::Matcher;
use grep_searcher::{Searcher, SearcherBuilder, Sink, SinkMatch};
use ignore::WalkBuilder;

use crate::config::Config;
use crate::error::SearchError;
use crate::matcher;
use crate::types::{Match, SubMatch};

pub(crate) fn search(config: &Config) -> Result<Vec<Match>, SearchError> {
    let matcher = matcher::build_regex(&config.pattern)?;
    let mut searcher = SearcherBuilder::new().line_number(true).build();

    let mut results = Vec::new();
    let mut builder = WalkBuilder::new(&config.paths[0]);
    for path in config.paths.iter().skip(1) {
        builder.add(path);
    }

    for entry in builder.build() {
        let entry = entry?;
        let is_file = entry
            .file_type()
            .map(|file_type| file_type.is_file())
            .unwrap_or(false);
        if !is_file {
            continue;
        }

        let path = entry.path().to_path_buf();
        let sink = CollectSink::new(&path, &matcher, &mut results);
        searcher.search_path(&matcher, &path, sink)?;
    }

    Ok(results)
}

struct CollectSink<'a, M: Matcher> {
    path: &'a Path,
    matcher: &'a M,
    results: &'a mut Vec<Match>,
}

impl<'a, M: Matcher> CollectSink<'a, M> {
    fn new(path: &'a Path, matcher: &'a M, results: &'a mut Vec<Match>) -> Self {
        Self {
            path,
            matcher,
            results,
        }
    }
}

impl<'a, M: Matcher> Sink for CollectSink<'a, M> {
    type Error = io::Error;

    fn matched(&mut self, _searcher: &Searcher, mat: &SinkMatch<'_>) -> Result<bool, Self::Error> {
        let bytes = mat.bytes();
        let mut submatches = Vec::new();
        self.matcher
            .find_iter(bytes, |m| {
                submatches.push(SubMatch {
                    start: m.start(),
                    end: m.end(),
                });
                true
            })
            .map_err(|err| io::Error::new(io::ErrorKind::Other, err.to_string()))?;

        let column = submatches.first().map(|m| m.start);
        let line_text = String::from_utf8_lossy(bytes).to_string();

        self.results.push(Match {
            path: self.path.to_path_buf(),
            line: mat.line_number(),
            column,
            bytes: bytes.to_vec(),
            submatches,
            line_text,
        });

        Ok(true)
    }
}
