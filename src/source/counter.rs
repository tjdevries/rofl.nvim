use super::Source;
use crate::{Entry, Score, SharedNvim};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct Counter(pub u64);

#[async_trait]
impl Source for Counter {
    async fn get(&mut self, _: SharedNvim, _user_match: &str) -> Vec<Entry> {
        let entry = Entry::new(format!("The counter is {}", self.0), Score::new(0));
        self.0 += 1;
        vec![entry]
    }
}
