use crate::error::SearchError;
use grep_regex::RegexMatcher;

pub(crate) fn build_regex(pattern: &str) -> Result<RegexMatcher, SearchError> {
    RegexMatcher::new(pattern).map_err(|err| SearchError::InvalidPattern(err.to_string()))
}
