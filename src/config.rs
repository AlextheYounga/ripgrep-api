use std::path::PathBuf;

#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub(crate) pattern: String,
    pub(crate) paths: Vec<PathBuf>,
}

impl Config {
    pub(crate) fn new(pattern: String) -> Self {
        let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            pattern,
            paths: vec![root],
        }
    }
}
