mod builder;
mod config;
mod engine;
mod error;
mod matcher;
mod search;
mod types;

pub use builder::SearchBuilder;
pub use error::SearchError;
pub use search::Search;
pub use types::{ContextLine, ContextKind, Match, SubMatch};

pub fn rg(pattern: impl Into<String>) -> SearchBuilder {
    SearchBuilder::new(pattern)
}
