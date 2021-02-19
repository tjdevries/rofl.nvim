// Erik recommends: https://tracing.rs/tracing/
use async_trait::async_trait;
use log::{error, info, LevelFilter};
use nvim_rs::{compat::tokio::Compat, create::tokio as create, Handler, Neovim, Value};
use simplelog::WriteLogger;
use sources::{BufferCompletionSource, CompletionSource, Completions, FileCompletionSource};
use std::{
    collections::HashMap,
    panic,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokio::{runtime, sync::RwLock};

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

    /// Current buffer
    bufnr: u64,
    // Enabled sources
    // sources: HashMap<SourceType, CompletionSource>,
    // sources: Vec<CompletionSource>,
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

fn lookup_u64_key(map: &Vec<(Value, Value)>, lookup_key: &str) -> u64 {
    map.iter()
        .find(|(key, _)| key.as_str().expect("string keys") == lookup_key)
        .expect("TJ is dumb and twitch chat is smart // u64 way")
        .1
        .as_u64()
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
        let bufnr = lookup_u64_key(&map, "bufnr");

        CompletionContext { word, cwd, bufnr }
    }
}

#[derive(Debug, Clone)]
pub struct SourceContext {
    /// Is file source enabled?
    file: bool,

    /// Is buffer source enabled?
    buffer: bool,
}

impl From<Vec<(Value, Value)>> for SourceContext {
    fn from(map: Vec<(Value, Value)>) -> Self {
        let coerce_bool = |index| match index {
            Value::Boolean(val) => val.clone(),
            Value::Nil => false,
            _ => false,
        };

        let mut file = false;
        let mut buffer = false;
        for (key, index) in map.iter() {
            let key = key.as_str().expect("keys are strings");
            if key == "file" {
                file = coerce_bool(index.clone());
            } else if key == "buffer" {
                buffer = coerce_bool(index.clone());
            }
        }

        SourceContext { file, buffer }
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

    Ok(Value::Nil)
}

#[async_trait]
impl Handler for NeovimHandler {
    type Writer = Compat<tokio::io::Stdout>;

    async fn handle_request(
        &self,
        name: String,
        args: Vec<Value>,
        _neovim: Neovim<Self::Writer>,
    ) -> Result<Value, Value> {
        info!("===========================================================");
        info!("Request: {}, {:?}", name, args);

        match name.as_ref() {
            "find_start" => {
                info!("======================= FIND START ==================================");
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
                info!("======================= COMPLETE ==================================");
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
                let map_context_value = args[0].as_map().expect("map_context").clone();
                let map_context = CompletionContext::from(map_context_value);
                info!("context: {:?}", map_context);

                let source_context_value = args[1].as_map().unwrap_or(&Vec::new()).clone();
                let source_context = SourceContext::from(source_context_value);

                let mut completions: Completions = Completions { items: Vec::new() };

                // TODO: Decide on mutability
                if source_context.file {
                    let completions_file = self.file_completion.complete(&map_context);
                    if let Ok(c) = completions_file {
                        info!("Adding file completions");
                        completions.items.extend(c.items)
                    }
                }

                if source_context.buffer {
                    let completions_buffer = self
                        .buffer_completion
                        .lock()
                        .expect("gets the lock")
                        .complete(&map_context);

                    if let Ok(c) = completions_buffer {
                        info!("Adding buffer completions");
                        completions.items.extend(c.items)
                    }
                }

                let array = Value::Array(
                    completions
                        .items
                        .iter()
                        .map(|x| Value::from(x.word.clone()))
                        .collect(),
                );

                info!("{:?}", completions);
                info!("Array: {:?}", &array);

                Ok(array)
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
                info!("Calling buf attach lines");

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

                info!(
                    "Completed buf attach lines {:?}",
                    self.buffer_completion.lock().expect("locked")
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
        buffer_completion: Arc::new(Mutex::new(BufferCompletionSource {
            word_store: HashMap::new(),
        })),
    })
    .await;

    let cache_path = dirs_next::cache_dir()
        .expect("Failed to get cache dir")
        .join("nvim");

    // should be okay to be synchronous
    std::fs::create_dir_all(&cache_path).expect("Failed to create cache dir");

    WriteLogger::init(
        LevelFilter::Debug,
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
