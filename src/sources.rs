// Resources:
//
// - https://rust-analyzer.github.io/rust-analyzer/ide/struct.CompletionItem.html
// - https://microsoft.github.io/language-server-protocol/specifications/specification-current/#textDocument_completion
//

use std::{fs, path::Path};

use crate::CompletionContext;
use anyhow::Result;
use log::info;

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
#[derive(Debug)]
pub struct CompletionItem {
    pub word: String,
}

pub trait CompletionSource {
    fn complete(&self, ctx: &CompletionContext) -> Result<Completions>;
}

// This completes filenames
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
                        info!("Examining Path: {:?}", path);

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
