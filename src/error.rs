use std::fmt;

#[derive(Debug)]
pub enum SearchError {
    InvalidPattern(String),
    InvalidGlob(String),
    InvalidType(String),
    Walk(ignore::Error),
    Io(std::io::Error),
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPattern(message) => write!(f, "invalid pattern: {message}"),
            Self::InvalidGlob(message) => write!(f, "invalid glob: {message}"),
            Self::InvalidType(message) => write!(f, "invalid type: {message}"),
            Self::Walk(err) => write!(f, "walk error: {err}"),
            Self::Io(err) => write!(f, "io error: {err}"),
        }
    }
}

impl std::error::Error for SearchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidPattern(_) | Self::InvalidGlob(_) | Self::InvalidType(_) => None,
            Self::Walk(err) => Some(err),
            Self::Io(err) => Some(err),
        }
    }
}

impl From<ignore::Error> for SearchError {
    fn from(err: ignore::Error) -> Self {
        Self::Walk(err)
    }
}

impl From<std::io::Error> for SearchError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}
