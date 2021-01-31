use super::Source;
use crate::{Entry, Score, SharedNvim};
use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct Counter(pub u64);

#[async_trait]
impl Source for Counter {
    async fn get(&mut self, _: SharedNvim, mut sender: Sender<Entry>, user_match: &str) {
        let entry = Entry::new(format!("The counter is {}", self.0), Score::new(0));
        if let Some(entry) = entry.score(&user_match) {
            sender.send(entry).await.unwrap();
        }
        self.0 += 1;
    }
}
