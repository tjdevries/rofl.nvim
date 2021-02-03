use super::Score;
use futures::stream::{self, StreamExt};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use nvim_rs::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub contents: String,
    pub score: Score,
}

impl Entry {
    pub fn new(contents: String, score: Score) -> Entry {
        Entry { contents, score }
    }

    pub async fn serialize(entries: Vec<Entry>) -> Value {
        Value::Array(stream::iter(entries).map(|e| e.into()).collect().await)
    }

    pub fn score(self, text: &str) -> Option<Entry> {
        let matcher = SkimMatcherV2::default();
        matcher
            .fuzzy_match(&self.contents, text)
            .map(|score| Entry::new(self.contents, Score::new(score)))
    }

    pub fn score_multiple(entries: Vec<Entry>, text: &str) -> Vec<Entry> {
        let mut scored: Vec<_> = entries.into_iter().filter_map(|e| e.score(text)).collect();
        scored.sort_unstable_by(|e1, e2| e1.score.cmp(&e2.score));
        scored
    }
}

impl From<Entry> for Value {
    fn from(entry: Entry) -> Value {
        Value::String(entry.contents.into())
    }
}
