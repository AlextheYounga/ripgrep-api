use crate::config::{CaseMode, Config};
use crate::error::SearchError;
use grep_regex::{RegexMatcher, RegexMatcherBuilder};

pub(crate) fn build_regex(pattern: &str, config: &Config) -> Result<RegexMatcher, SearchError> {
    let mut builder = RegexMatcherBuilder::new();

    match config.case_mode {
        CaseMode::Smart => {
            builder.case_smart(true);
        }
        CaseMode::Insensitive => {
            builder.case_insensitive(true);
        }
        CaseMode::Sensitive => {}
    }

    if config.fixed_strings {
        builder.fixed_strings(true);
    }

    if config.word {
        builder.word(true);
    }

    if config.line_regexp {
        builder.whole_line(true);
    }

    builder
        .build(pattern)
        .map_err(|err| SearchError::InvalidPattern(err.to_string()))
}
