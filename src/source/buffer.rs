use super::Source;
use crate::{Entry, Score, SharedNvim};
use anyhow::Result;
use async_trait::async_trait;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct BufferWords {
    words: Vec<String>,
}

impl BufferWords {
    pub fn new() -> BufferWords {
        BufferWords { words: Vec::new() }
    }
}

#[async_trait]
impl Source for BufferWords {
    async fn get(&mut self, nvim: SharedNvim, _user_match: &str) -> Vec<Entry> {
        self.words
            .clone()
            .into_iter()
            .map(|s| Entry::new(s, Score::new(0)))
            .collect()
    }

    async fn update(&mut self, nvim: SharedNvim) -> Result<()> {
        let lines = nvim.get_current_buf().await?.get_lines(0, -1, true).await?;
        let re = Regex::new(r"\w+").unwrap(); // TODO: use Tj's iskeyword 
        let words: Vec<_> = lines
            .into_iter()
            .map(|line| {
                re.find_iter(&line)
                    .map(|m| m.as_str().to_string())
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();
        self.words = words;
        Ok(())
    }
}
