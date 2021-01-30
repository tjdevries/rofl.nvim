use super::Source;
use crate::{Entry, Nvim, Score};
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct Counter(pub u64);

#[async_trait]
impl Source for Counter {
    async fn get(&mut self, _: Nvim) -> Vec<Entry> {
        let res = vec![Entry::new(
            format!("The counter is {}", self.0),
            Score::new(0),
        )];
        self.0 += 1;
        res
    }
}
