use crate::config::{CaseMode, Config, RegexEngine};
use crate::error::SearchError;
use grep_matcher::{Match, Matcher, NoCaptures};
use grep_regex::{RegexMatcher, RegexMatcherBuilder};

#[cfg(feature = "pcre2")]
use grep_pcre2::{RegexMatcher as Pcre2Matcher, RegexMatcherBuilder as Pcre2MatcherBuilder};

pub(crate) fn build_matcher(pattern: &str, config: &Config) -> Result<EngineMatcher, SearchError> {
    match config.engine {
        RegexEngine::Default => build_default_matcher(pattern, config),
        #[cfg(feature = "pcre2")]
        RegexEngine::Pcre2 => build_pcre2_matcher(pattern, config),
    }
}

fn apply_case(builder: &mut RegexMatcherBuilder, mode: CaseMode) {
    match mode {
        CaseMode::Smart => {
            builder.case_smart(true);
        }
        CaseMode::Insensitive => {
            builder.case_insensitive(true);
        }
        CaseMode::Sensitive => {}
    }
}

fn build_default_matcher(pattern: &str, config: &Config) -> Result<EngineMatcher, SearchError> {
    let mut builder = RegexMatcherBuilder::new();
    apply_case(&mut builder, config.case_mode);
    if config.fixed_strings {
        builder.fixed_strings(true);
    }
    if config.word {
        builder.word(true);
    }
    if config.line_regexp {
        builder.whole_line(true);
    }

    let matcher = builder
        .build(pattern)
        .map_err(|err| SearchError::InvalidPattern(err.to_string()))?;
    Ok(EngineMatcher::Regex(matcher))
}

#[cfg(feature = "pcre2")]
fn build_pcre2_matcher(pattern: &str, config: &Config) -> Result<EngineMatcher, SearchError> {
    let mut builder = Pcre2MatcherBuilder::new();
    match config.case_mode {
        CaseMode::Smart => {
            builder.case_smart(true);
        }
        CaseMode::Insensitive => {
            builder.caseless(true);
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

    let matcher = builder
        .build(pattern)
        .map_err(|err| SearchError::InvalidPattern(err.to_string()))?;
    Ok(EngineMatcher::Pcre2(matcher))
}

pub(crate) enum EngineMatcher {
    Regex(RegexMatcher),
    #[cfg(feature = "pcre2")]
    Pcre2(Pcre2Matcher),
}

impl Matcher for EngineMatcher {
    type Captures = NoCaptures;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn find_at(&self, haystack: &[u8], at: usize) -> Result<Option<Match>, Self::Error> {
        match self {
            EngineMatcher::Regex(matcher) => matcher
                .find_at(haystack, at)
                .map_err(|err| Box::new(err) as _),
            #[cfg(feature = "pcre2")]
            EngineMatcher::Pcre2(matcher) => matcher
                .find_at(haystack, at)
                .map_err(|err| Box::new(err) as _),
        }
    }

    fn new_captures(&self) -> Result<Self::Captures, Self::Error> {
        Ok(NoCaptures::new())
    }

    fn capture_count(&self) -> usize {
        0
    }

    fn capture_index(&self, _name: &str) -> Option<usize> {
        None
    }

    fn find(&self, haystack: &[u8]) -> Result<Option<Match>, Self::Error> {
        match self {
            EngineMatcher::Regex(matcher) => {
                matcher.find(haystack).map_err(|err| Box::new(err) as _)
            }
            #[cfg(feature = "pcre2")]
            EngineMatcher::Pcre2(matcher) => {
                matcher.find(haystack).map_err(|err| Box::new(err) as _)
            }
        }
    }

    fn shortest_match(&self, haystack: &[u8]) -> Result<Option<usize>, Self::Error> {
        match self {
            EngineMatcher::Regex(matcher) => matcher
                .shortest_match(haystack)
                .map_err(|err| Box::new(err) as _),
            #[cfg(feature = "pcre2")]
            EngineMatcher::Pcre2(matcher) => matcher
                .shortest_match(haystack)
                .map_err(|err| Box::new(err) as _),
        }
    }
}
