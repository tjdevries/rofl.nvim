// Resources:
//
// - https://rust-analyzer.github.io/rust-analyzer/ide/struct.CompletionItem.html
// - https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocument_completion
//

use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use crate::CompletionContext;
use anyhow::Result;
use log::{info, trace};

// CompletionSource: function(ctx) -> Completions
//
// FileCompletionSource implements CompletionSource

#[derive(Debug)]
pub struct Completions {
    pub items: Vec<CompletionItem>,
}

///                         *complete-items*
/// Each list item can either be a string or a Dictionary.  When it is a string it
/// is used as the completion.  When it is a Dictionary it can contain these
/// items:
///     word        the text that will be inserted, mandatory
///
///     abbr        abbreviation of "word"; when not empty it is used in
///                 the menu instead of "word"
///
///     menu        extra text for the popup menu, displayed after "word"
///                 or "abbr"
///
///     info        more information about the item, can be displayed in a
///                 preview window
///
///     kind        single letter indicating the type of completion
///
///     icase       when non-zero case is to be ignored when comparing
///                 items to be equal; when omitted zero is used, thus
///                 items that only differ in case are added
///
///     equal       when non-zero, always treat this item to be equal when
///                 comparing. Which means, "equal=1" disables filtering
///                 of this item.
///
///     dup         when non-zero this match will be added even when an
///                 item with the same word is already present.
///
///     empty       when non-zero this match will be added even when it is
///                 an empty string
///
///     user_data   custom data which is associated with the item and
///                 available in |v:completed_item|; it can be any type;
///                 defaults to an empty string
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub word: String,
}

pub trait CompletionSource {
    fn complete(&self, ctx: &CompletionContext) -> Result<Completions>;

    fn on_lines(&mut self, _bufnr: u64, _start_line: u64, _final_line: u64, _lines: &Vec<String>) {}
}

// This completes filenames
#[derive(Debug, Clone)]
pub struct FileCompletionSource;

impl CompletionSource for FileCompletionSource {
    fn complete(&self, ctx: &CompletionContext) -> Result<Completions> {
        let path_to_complete = Path::new(ctx.word.as_str());

        // TODO: Definitely not handling all the cases.
        // "/hello/world" -> "/hello"
        // "README.m" -> $CWD
        let path_tail = path_to_complete.file_name();
        let mut path_parent = path_to_complete.parent().unwrap_or(&ctx.cwd);
        if path_parent == Path::new("") {
            path_parent = &ctx.cwd;
        }
        info!(
            "To Complete: {:?}, Path Parent: {:?}",
            path_to_complete, path_parent
        );

        // TODO: Use tokio's FS stuff so that I'm async :)
        // let mut results: Vec<CompletionItem> = Vec::new()
        // for entry in fs::read_dir(path_parent)? {
        //     let entry = entry?;
        //     let path = entry.path();

        //     results.push(CompletionItem {
        //         word: String::from("hello")
        //     })
        // }

        Ok(Completions {
            items: fs::read_dir(path_parent)?
                .filter_map(|entry| {
                    entry.map_or(None, |x| {
                        let path = x.path();
                        trace!("Examining Path: {:?}", path);

                        if let Some(path_filter) = path_tail {
                            if let Some(tail) = path.file_name() {
                                let tail = tail.to_str().expect("can make a string");
                                let path_filter = path_filter.to_str().expect("can make str");

                                if !tail.starts_with(path_filter) {
                                    return None;
                                }
                            }
                        }

                        let relative_path = pathdiff::diff_paths(&path, path_parent)?;
                        Some(CompletionItem {
                            word: String::from(relative_path.to_str().expect("Can make a str")),
                        })
                    })
                })
                .collect(),
        })
        // Ok(Completions { items: Vec::new() })
        // if let Ok(iter_dir) = fs::read_dir(path_parent) {
        // } else {
        //     return Completions { items: Vec::new() };
        // }

        // if ctx.word == "README.m" {
        //     let mut items = Vec::new();
        //     items.push(CompletionItem {
        //         word: "README.md".to_string(),
        //     });
        //     Completions { items }
        // } else {
        //     Completions { items: Vec::new() }
        // }
    }
}

#[derive(Debug, Clone)]
pub struct BufferWordStore {
    lines_to_words: HashMap<u64, Vec<String>>,

    // TODO: Later it would be cool to do this with a Trie
    words: HashMap<String, u64>,
}

impl Default for BufferWordStore {
    fn default() -> Self {
        Self {
            lines_to_words: HashMap::new(),
            words: HashMap::new(),
        }
    }
}

#[allow(dead_code)]
impl BufferWordStore {
    pub fn update(&mut self, line: u64, words: Vec<String>) {
        let removed_words = match self.lines_to_words.get(&line) {
            Some(original_words) => original_words
                .iter()
                .filter(|word| !words.contains(word))
                .collect::<Vec<&String>>(),
            _ => Vec::new(),
        };

        // then add the count for the new words
        // ??? profit

        // OK, decrement the count in words for each of the words we're missing
        for word in &removed_words {
            *self.words.get_mut(*word).unwrap() -= 1;
        }

        for word in removed_words.into_iter() {
            if let Some(count) = self.words.get(word) {
                if *count == 0 {
                    self.words.remove(word);
                }
            }
        }

        // TODO: Seems like this should be one line in rust.
        // Probably twice as many characters and 30 more functions, but could be one line.
        for word in &words {
            self.words.entry(word.clone()).or_insert(0);
            *self.words.get_mut(word).unwrap() += 1;
        }

        self.lines_to_words.insert(line, words.clone());
    }

    pub fn get_exact_matches(&self, prefix: &str) -> HashSet<String> {
        let mut result = HashSet::new();
        for (word, _) in &self.words {
            if word.starts_with(prefix) {
                result.insert(word.to_owned());
            }
        }

        result
    }
}

/// Completes words in open buffers
///
/// Has an `on_bytes` / `on_lines` callback to update the state
/// of the words in open buffers.
///
/// This is super overkill and that's OK :) I just wanna learn Rust.
#[derive(Debug, Clone)]
pub struct BufferCompletionSource {
    pub word_store: HashMap<u64, BufferWordStore>,
}

impl CompletionSource for BufferCompletionSource {
    fn complete(&self, ctx: &CompletionContext) -> Result<Completions> {
        match self.word_store.get(&ctx.bufnr) {
            None => Ok(Completions { items: Vec::new() }),
            Some(buffer_word_store) => Ok(Completions {
                // items: vec![CompletionItem {
                //     word: String::from("hello"),
                // }],
                items: buffer_word_store
                    .get_exact_matches(&ctx.word)
                    .into_iter()
                    .map(|x| CompletionItem { word: x })
                    .collect(),
            }),
        }
    }

    fn on_lines(&mut self, bufnr: u64, start_line: u64, _final_line: u64, lines: &Vec<String>) {
        let buffer_word_store = self
            .word_store
            .entry(bufnr)
            .or_insert(BufferWordStore::default());

        for index in 0..lines.len() as u64 {
            let line = start_line + index;
            let text: Vec<String> = lines[index as usize]
                .clone()
                .split(" ")
                .map(|x| x.to_string())
                .collect();

            buffer_word_store.update(line, text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_updater() {
        let mut buffer_store = BufferWordStore::default();

        buffer_store.update(1, vec![String::from("hello"), String::from("world")]);
        buffer_store.update(2, vec![String::from("world")]);

        assert_eq!(HashSet::new(), buffer_store.get_exact_matches("asdf"));

        let mut hello_match = HashSet::new();
        hello_match.insert(String::from("hello"));
        assert_eq!(hello_match, buffer_store.get_exact_matches("hel"));
    }

    #[test]
    fn test_removing_line() {
        let mut buffer_store = BufferWordStore::default();

        buffer_store.update(1, vec![String::from("hello"), String::from("world")]);
        buffer_store.update(2, vec![String::from("world")]);
        buffer_store.update(1, vec![String::from("world")]);

        dbg!(&buffer_store);

        assert_eq!(HashSet::new(), buffer_store.get_exact_matches("hel"));
    }
}
