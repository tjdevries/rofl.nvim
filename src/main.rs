// Erik recommends: https://tracing.rs/tracing/
use async_trait::async_trait;
use log::{error, info, LevelFilter};
use nvim_rs::{compat::tokio::Compat, create::tokio as create, Handler, Neovim, Value};
use simplelog::WriteLogger;
use sources::{BufferCompletionSource, CompletionSource, FileCompletionSource};
use std::{
    collections::HashMap,
    panic,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio::{io::Stdout, runtime, sync::RwLock};

mod collections;
mod nvim;
mod sources;

use nvim::iskeyword;

#[derive(Debug)]
pub struct CompletionContext {
    /// The word under the cursor
    word: String,

    /// Current working directory for Neovim
    cwd: PathBuf,
}

fn lookup_str_key(map: &Vec<(Value, Value)>, lookup_key: &str) -> String {
    map.iter()
        .find(|(key, _)| key.as_str().expect("string keys") == lookup_key)
        .expect("TJ is dumb and twitch chat is smart")
        .1
        .as_str()
        .expect("Did I send things gud")
        .to_owned()
}

impl From<Vec<(Value, Value)>> for CompletionContext {
    fn from(map: Vec<(Value, Value)>) -> Self {
        // I've got a vector of value value, which is key:value pairs
        // I know the names of the values I want
        // ....
        //
        // local ctx = {}
        // for k, v in pairs(request) do ctx[k] = v end
        // return ctx
        //
        //
        // local word = nil
        // for k, v in pairs(request) do if k == "word" then word = v end end

        let word = lookup_str_key(&map, "word");
        let cwd: PathBuf = Path::new(lookup_str_key(&map, "cwd").as_str()).into();

        CompletionContext { word, cwd }
    }
}

#[derive(Debug, Clone)]
struct NeovimHandler {
    iskeyword_map: Arc<RwLock<HashMap<u64, iskeyword::KeywordMatcher>>>,

    file_completion: FileCompletionSource,
    buffer_completion: Arc<Mutex<BufferCompletionSource>>,
}

async fn buf_initialize(handler: &NeovimHandler, args: Vec<Value>) -> Result<Value, Value> {
    let mut iskeyword_map = handler.iskeyword_map.write().await;

    let bufnr = args[0].as_u64().expect("Yo dawg, send me those bufnrs");
    let iskeyword_str = args[1].as_str().expect("iskeyword to be sent as a string");

    // info!("{:?}", nvim::iskeyword::transform(&iskeyword_str));

    info!("old iskeyword {:?}", iskeyword_map);
    iskeyword_map.insert(bufnr, iskeyword::transform(iskeyword_str));
    info!("new iskeyword {:?}", iskeyword_map);

    return Ok(Value::from("hello"));
}

#[async_trait]
impl Handler for NeovimHandler {
    type Writer = Compat<Stdout>;

    async fn handle_request(
        &self,
        name: String,
        args: Vec<Value>,
        _neovim: Neovim<Self::Writer>,
    ) -> Result<Value, Value> {
        info!("Request: {}, {:?}", name, args);

        match name.as_ref() {
            "find_start" => {
                let current_bufnr = args[0].as_u64().expect("Bufnr");
                let current_line = args[1].to_string();
                let current_cursor = args[2].as_u64().expect("Should get a number");

                let iskeyword_map = self.iskeyword_map.read().await;

                let iskeyword_option = iskeyword_map.get(&current_bufnr);
                if iskeyword_option.is_none() {
                    return Ok(Value::from(-1));
                }

                let keyword_matcher = iskeyword_option.expect("Has to be it now");
                let line_range = keyword_matcher.find(&current_line, current_cursor);
                let current_slice = &current_line.to_string()[line_range.start..line_range.finish];

                info!("find_start: {}, {:?}", current_cursor, current_slice);

                // minus 1 because indexing
                Ok(Value::from(line_range.start - 1))
            }
            "complete" => {
                info!("Completing...");
                Ok(Value::Array(vec![Value::from("hello")]))
            }
            "complete_sync" => {
                // TODO: Read about:
                // - try_unpack
                // - move vs clone
                // - Results
                // - Option vs Result
                //
                // - Other:
                //  - iterators
                //  - iter() vs into_iter() vs iter_mut()
                //  - collect()
                //
                //  - https://upsuper.github.io/rust-cheatsheet/?dark
                //
                //  - interior mutability
                //  - RefCel
                //
                // let map_context = args[0].as_map().expect("map_context").clone();
                // let map_context: Vec<(Value, Value)> = args[0].try_unpack().expect("map_context");
                // let map_context = args[0].as_map().iter.map(|(key, value)| ...);
                let map_context = args[0].as_map().expect("map_context").clone();
                let context = CompletionContext::from(map_context);
                info!("context: {:?}", context);

                // TODO: Decide on mutability
                let completions = self.file_completion.complete(&context);

                Ok(Value::Array(match completions {
                    Ok(completions) => completions
                        .items
                        .iter()
                        .map(|x| Value::from(&x.word[..]))
                        .collect(),
                    Err(_) => vec![],
                }))

                // let neovim_stuff: Vec<Value> = completions
                //     .items
                //     .iter()
                //     .map(|x| Value::from(&x.word[..]))
                //     .collect();

                // Ok(Value::Array(neovim_stuff))
            }
            "buf_initialize" => {
                return buf_initialize(self, args).await;
            }
            _ => Ok(Value::from(3)),
        }
    }

    async fn handle_notify(&self, name: String, args: Vec<Value>, _neovim: Neovim<Self::Writer>) {
        match name.as_ref() {
            "complete" => {}
            "v_char" => {}
            "insert_leave" => {}
            "buf_initialize" => {
                let _ = buf_initialize(self, args).await;
            }
            "buf_attach_lines" => {
                // Call all the `on_attach` methods for existing sources.
                let bufnr = args[0].as_u64().expect("bufnr");
                let start_line = args[1].as_u64().expect("start_line");
                let final_line = args[2].as_u64().expect("final_line");
                let resulting_lines: Vec<String> = args[3]
                    .as_array()
                    .expect("resulting_lines")
                    .iter()
                    .map(|val| val.as_str().expect("Sent strings").to_string())
                    .collect();

                self.buffer_completion.lock().expect("locked").on_lines(
                    bufnr,
                    start_line,
                    final_line,
                    &resulting_lines,
                );
            }
            _ => (),
        }
    }
}

async fn run() {
    let (nvim, io_handler) = create::new_parent(NeovimHandler {
        iskeyword_map: Arc::new(RwLock::new(HashMap::new())),

        // TODO: We should actually make it so that we have some hashmap of
        // completion source names -> completion sources.
        //
        // This way we can just register and add them as we go
        //
        // Then you can only request from sources, etc.
        file_completion: FileCompletionSource {},
        buffer_completion: Arc::new(Mutex::new(BufferCompletionSource {})),
    })
    .await;

    let cache_path = dirs_next::cache_dir()
        .expect("Failed to get cache dir")
        .join("nvim");

    // should be okay to be synchronous
    std::fs::create_dir_all(&cache_path).expect("Failed to create cache dir");

    WriteLogger::init(
        LevelFilter::Trace,
        simplelog::Config::default(),
        std::fs::File::create(cache_path.join("rofl.log")).expect("Failed to create log file"),
    )
    .expect("Failed to start logger");

    // we do not want to crash when panicking, instead log it
    panic::set_hook(Box::new(move |panic| {
        error!("----- Panic -----");
        error!("{}", panic);
    }));

    // TODO: Any error should probably be logged, as stderr is not visible to users.
    match io_handler.await {
        Ok(res) => {
            info!("OK Result: {:?}", res);
        }
        Err(err) => {
            nvim.err_writeln(&format!("Error: '{}'", err))
                .await
                .unwrap_or_else(|e| {
                    // We could inspect this error to see what was happening, and
                    // maybe retry, but at this point it's probably best
                    // to assume the worst and print a friendly and
                    // supportive message to our users
                    eprintln!("Well, dang... '{}'", e);
                });
        }
    }
}

fn main() {
    let mut runtime = runtime::Builder::new()
        .threaded_scheduler()
        .build()
        .expect("Failed to build runtime");
    runtime.block_on(run())
}
