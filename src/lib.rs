#![doc = include_str!("../README.md")]

mod builder;
mod config;
mod engine;
mod error;
mod matcher;
mod search;
mod sink;
mod types;

pub use builder::SearchBuilder;
pub use error::SearchError;
pub use search::Search;
pub use sink::MatchSink;
pub use types::{ContextKind, ContextLine, Match, SubMatch};

/// Create a new SearchBuilder with rg-style defaults.
pub fn rg(pattern: impl Into<String>) -> SearchBuilder {
    SearchBuilder::new(pattern)
}
