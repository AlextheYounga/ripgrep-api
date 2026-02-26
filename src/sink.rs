use crate::types::{ContextLine, Match};

pub trait MatchSink {
    fn matched(&mut self, mat: &Match) -> bool;

    fn context(&mut self, _line: &ContextLine) -> bool {
        true
    }

    fn finish(&mut self) {}
}
