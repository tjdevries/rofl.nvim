use super::Score;
use nvim_rs::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    contents: String,
    score: Score,
}

impl Entry {
    pub fn new(contents: String, score: Score) -> Entry {
        Entry { contents, score }
    }

    pub fn serialize(entries: Vec<Entry>) -> Value {
        Value::Array(entries.into_iter().map(|e| e.into()).collect())
    }
}

impl From<Entry> for Value {
    fn from(entry: Entry) -> Value {
        Value::String(entry.contents.into())
    }
}
