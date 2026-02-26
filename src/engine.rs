use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};

use grep_matcher::Matcher;
use grep_searcher::{BinaryDetection, Searcher, SearcherBuilder, Sink, SinkMatch};
use ignore::{overrides::OverrideBuilder, types::TypesBuilder, WalkBuilder};

use crate::config::Config;
use crate::error::SearchError;
use crate::matcher::{self, EngineMatcher};
use crate::sink::MatchSink;
use crate::types::{ContextKind, ContextLine, Match, SubMatch};

pub(crate) fn search(config: &Config) -> Result<Vec<Match>, SearchError> {
    let matcher = matcher::build_matcher(&config.pattern, config)?;
    let mut searcher = build_searcher(config);
    let mut results = Vec::new();

    for entry in build_walker(config)?.build() {
        let entry = entry?;
        if !is_file_entry(&entry) {
            continue;
        }

        let path = entry.path().to_path_buf();
        let sink = CollectSink::new(&path, &matcher, &mut results, config.max_count);
        searcher.search_path(&matcher, &path, sink)?;
    }

    Ok(results)
}

pub(crate) fn search_with<S: MatchSink>(config: &Config, sink: &mut S) -> Result<(), SearchError> {
    let matcher = matcher::build_matcher(&config.pattern, config)?;
    let mut searcher = build_searcher(config);

    for entry in build_walker(config)?.build() {
        let entry = entry?;
        if !is_file_entry(&entry) {
            continue;
        }

        let path = entry.path().to_path_buf();
        let mut callback = CallbackSink::new(&path, &matcher, sink, config.max_count);
        searcher.search_path(&matcher, &path, &mut callback)?;
    }

    Ok(())
}

pub(crate) fn search_reader<R: io::Read>(
    config: &Config,
    reader: R,
    source: &Path,
) -> Result<Vec<Match>, SearchError> {
    let matcher = matcher::build_matcher(&config.pattern, config)?;
    let mut searcher = build_searcher(config);
    let mut results = Vec::new();
    let sink = CollectSink::new(source, &matcher, &mut results, config.max_count);
    searcher.search_reader(&matcher, reader, sink)?;
    Ok(results)
}

pub(crate) fn search_reader_with<R: io::Read, S: MatchSink>(
    config: &Config,
    reader: R,
    source: &Path,
    sink: &mut S,
) -> Result<(), SearchError> {
    let matcher = matcher::build_matcher(&config.pattern, config)?;
    let mut searcher = build_searcher(config);
    let mut callback = CallbackSink::new(source, &matcher, sink, config.max_count);
    searcher.search_reader(&matcher, reader, &mut callback)?;
    Ok(())
}

pub(crate) fn search_slice(
    config: &Config,
    slice: &[u8],
    source: &Path,
) -> Result<Vec<Match>, SearchError> {
    let matcher = matcher::build_matcher(&config.pattern, config)?;
    let mut searcher = build_searcher(config);
    let mut results = Vec::new();
    let sink = CollectSink::new(source, &matcher, &mut results, config.max_count);
    searcher.search_slice(&matcher, slice, sink)?;
    Ok(results)
}

pub(crate) fn search_slice_with<S: MatchSink>(
    config: &Config,
    slice: &[u8],
    source: &Path,
    sink: &mut S,
) -> Result<(), SearchError> {
    let matcher = matcher::build_matcher(&config.pattern, config)?;
    let mut searcher = build_searcher(config);
    let mut callback = CallbackSink::new(source, &matcher, sink, config.max_count);
    searcher.search_slice(&matcher, slice, &mut callback)?;
    Ok(())
}

pub(crate) fn count(config: &Config) -> Result<u64, SearchError> {
    let matcher = matcher::build_matcher(&config.pattern, config)?;
    let mut searcher = build_searcher(config);
    let mut total = 0_u64;

    for entry in build_walker(config)?.build() {
        let entry = entry?;
        if !is_file_entry(&entry) {
            continue;
        }

        let path = entry.path().to_path_buf();
        let mut sink = CountSink::new(config.max_count);
        searcher.search_path(&matcher, &path, &mut sink)?;
        total = total.saturating_add(sink.count());
    }

    Ok(total)
}

pub(crate) fn files_with_matches(config: &Config) -> Result<Vec<PathBuf>, SearchError> {
    let matcher = matcher::build_matcher(&config.pattern, config)?;
    let mut searcher = build_searcher(config);
    let mut files = BTreeSet::new();

    for entry in build_walker(config)?.build() {
        let entry = entry?;
        if !is_file_entry(&entry) {
            continue;
        }

        let path = entry.path().to_path_buf();
        let mut sink = FirstMatchSink::new();
        searcher.search_path(&matcher, &path, &mut sink)?;
        if sink.found() {
            files.insert(path);
        }
    }

    Ok(files.into_iter().collect())
}

fn build_searcher(config: &Config) -> Searcher {
    let mut builder = SearcherBuilder::new();
    builder.line_number(true);
    builder.before_context(config.before_context);
    builder.after_context(config.after_context);
    if let Some(choice) = config.memory_map.clone() {
        builder.memory_map(choice);
    }
    if let Some(limit) = config.heap_limit {
        builder.heap_limit(Some(limit));
    }
    if config.binary_detection {
        builder.binary_detection(BinaryDetection::quit(b'\x00'));
    } else {
        builder.binary_detection(BinaryDetection::none());
    }
    builder.build()
}

fn build_walker(config: &Config) -> Result<WalkBuilder, SearchError> {
    let mut builder = WalkBuilder::new(&config.paths[0]);
    for path in config.paths.iter().skip(1) {
        builder.add(path);
    }

    builder
        .max_depth(config.max_depth)
        .max_filesize(config.max_filesize)
        .follow_links(config.follow_links)
        .hidden(!config.search_hidden)
        .parents(config.ignore_parent)
        .ignore(config.ignore_files)
        .git_ignore(config.ignore_vcs)
        .git_global(config.ignore_vcs)
        .git_exclude(config.ignore_vcs);

    if let Some(threads) = config.threads {
        builder.threads(threads);
    }

    if let Some(overrides) = &config.overrides {
        builder.overrides(overrides.clone());
    } else if !config.globs.is_empty() {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let mut overrides = OverrideBuilder::new(cwd);
        for glob in &config.globs {
            overrides
                .add(glob)
                .map_err(|err| SearchError::InvalidGlob(err.to_string()))?;
        }
        builder.overrides(
            overrides
                .build()
                .map_err(|err| SearchError::InvalidGlob(err.to_string()))?,
        );
    }

    if let Some(types) = &config.types_override {
        builder.types(types.clone());
    } else if !config.types.is_empty()
        || !config.type_not.is_empty()
        || !config.type_defs.is_empty()
    {
        let mut types = TypesBuilder::new();
        types.add_defaults();
        for (name, glob) in &config.type_defs {
            types
                .add(name, glob)
                .map_err(|err| SearchError::InvalidType(err.to_string()))?;
        }
        for name in &config.types {
            types.select(name);
        }
        for name in &config.type_not {
            types.negate(name);
        }
        builder.types(
            types
                .build()
                .map_err(|err| SearchError::InvalidType(err.to_string()))?,
        );
    }

    Ok(builder)
}

fn is_file_entry(entry: &ignore::DirEntry) -> bool {
    entry
        .file_type()
        .map(|file_type| file_type.is_file())
        .unwrap_or(false)
}

type DynMatcher = EngineMatcher;

struct CollectSink<'a> {
    path: &'a Path,
    matcher: &'a DynMatcher,
    results: &'a mut Vec<Match>,
    pending_before: Vec<ContextLine>,
    last_match_index: Option<usize>,
    max_count: Option<usize>,
    match_count: usize,
}

impl<'a> CollectSink<'a> {
    fn new(
        path: &'a Path,
        matcher: &'a DynMatcher,
        results: &'a mut Vec<Match>,
        max_count: Option<usize>,
    ) -> Self {
        Self {
            path,
            matcher,
            results,
            pending_before: Vec::new(),
            last_match_index: None,
            max_count,
            match_count: 0,
        }
    }
}

impl<'a> Sink for CollectSink<'a> {
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

        let column = submatches.first().map(|m| m.start.saturating_add(1));
        let line_text = String::from_utf8_lossy(bytes).to_string();
        let context = std::mem::take(&mut self.pending_before);

        self.results.push(Match {
            path: self.path.to_path_buf(),
            line: mat.line_number(),
            column,
            bytes: bytes.to_vec(),
            submatches,
            line_text,
            context,
        });

        self.last_match_index = Some(self.results.len().saturating_sub(1));
        self.match_count = self.match_count.saturating_add(1);
        if let Some(max_count) = self.max_count {
            if self.match_count >= max_count {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn context(
        &mut self,
        _searcher: &Searcher,
        context: &grep_searcher::SinkContext<'_>,
    ) -> Result<bool, Self::Error> {
        let kind = match context.kind() {
            grep_searcher::SinkContextKind::Before => ContextKind::Before,
            grep_searcher::SinkContextKind::After => ContextKind::After,
            grep_searcher::SinkContextKind::Other => ContextKind::Other,
        };
        let line = ContextLine {
            path: self.path.to_path_buf(),
            kind,
            line: context.line_number(),
            bytes: context.bytes().to_vec(),
            line_text: String::from_utf8_lossy(context.bytes()).to_string(),
        };

        match kind {
            ContextKind::Before => self.pending_before.push(line),
            ContextKind::After | ContextKind::Other => {
                if let Some(index) = self.last_match_index {
                    if let Some(existing) = self.results.get_mut(index) {
                        existing.context.push(line);
                    }
                }
            }
        }

        Ok(true)
    }
}

struct CountSink {
    count: u64,
    max_count: Option<usize>,
}

impl CountSink {
    fn new(max_count: Option<usize>) -> Self {
        Self {
            count: 0,
            max_count,
        }
    }

    fn count(&self) -> u64 {
        self.count
    }
}

impl Sink for CountSink {
    type Error = io::Error;

    fn matched(&mut self, _searcher: &Searcher, _mat: &SinkMatch<'_>) -> Result<bool, Self::Error> {
        self.count = self.count.saturating_add(1);
        if let Some(max_count) = self.max_count {
            if self.count >= max_count as u64 {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

struct CallbackSink<'a, S: MatchSink> {
    path: &'a Path,
    matcher: &'a DynMatcher,
    sink: &'a mut S,
    max_count: Option<usize>,
    match_count: usize,
}

impl<'a, S: MatchSink> CallbackSink<'a, S> {
    fn new(
        path: &'a Path,
        matcher: &'a DynMatcher,
        sink: &'a mut S,
        max_count: Option<usize>,
    ) -> Self {
        Self {
            path,
            matcher,
            sink,
            max_count,
            match_count: 0,
        }
    }
}

impl<'a, S: MatchSink> Sink for CallbackSink<'a, S> {
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

        let column = submatches.first().map(|m| m.start.saturating_add(1));
        let line_text = String::from_utf8_lossy(bytes).to_string();
        let mat = Match {
            path: self.path.to_path_buf(),
            line: mat.line_number(),
            column,
            bytes: bytes.to_vec(),
            submatches,
            line_text,
            context: Vec::new(),
        };

        self.match_count = self.match_count.saturating_add(1);
        let mut keep_going = self.sink.matched(&mat);
        if let Some(max_count) = self.max_count {
            if self.match_count >= max_count {
                keep_going = false;
            }
        }
        Ok(keep_going)
    }

    fn context(
        &mut self,
        _searcher: &Searcher,
        context: &grep_searcher::SinkContext<'_>,
    ) -> Result<bool, Self::Error> {
        let kind = match context.kind() {
            grep_searcher::SinkContextKind::Before => ContextKind::Before,
            grep_searcher::SinkContextKind::After => ContextKind::After,
            grep_searcher::SinkContextKind::Other => ContextKind::Other,
        };
        let line = ContextLine {
            path: self.path.to_path_buf(),
            kind,
            line: context.line_number(),
            bytes: context.bytes().to_vec(),
            line_text: String::from_utf8_lossy(context.bytes()).to_string(),
        };

        Ok(self.sink.context(&line))
    }

    fn finish(
        &mut self,
        _searcher: &Searcher,
        _finish: &grep_searcher::SinkFinish,
    ) -> Result<(), Self::Error> {
        self.sink.finish();
        Ok(())
    }
}

struct FirstMatchSink {
    found: bool,
}

impl FirstMatchSink {
    fn new() -> Self {
        Self { found: false }
    }

    fn found(&self) -> bool {
        self.found
    }
}

impl Sink for FirstMatchSink {
    type Error = io::Error;

    fn matched(&mut self, _searcher: &Searcher, _mat: &SinkMatch<'_>) -> Result<bool, Self::Error> {
        self.found = true;
        Ok(false)
    }
}
