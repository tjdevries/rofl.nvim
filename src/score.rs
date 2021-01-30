#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Score(i64);

impl Score {
    pub fn new(n: i64) -> Score {
        Score(n)
    }
}
