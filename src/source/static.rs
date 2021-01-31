use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

use super::{Score, SharedNvim, Source};
use crate::Entry;

#[derive(Debug, Clone)]
pub struct Static(Vec<Entry>);

impl Static {
    pub fn new(entries: &[String]) -> Static {
        Static(
            entries
                .iter()
                .map(|s| Entry::new(String::from(s), Score::new(0)))
                .collect(),
        )
    }
}

#[async_trait]
impl Source for Static {
    async fn get(&mut self, _nvim: SharedNvim, mut sender: Sender<Entry>, user_match: &str) {
        let entries = self.0.clone();
        for entry in Entry::score_multiple(entries, user_match) {
            sender.send(entry.clone()).await.unwrap();
        }
    }
}
