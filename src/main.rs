// Erik recommends: https://tracing.rs/tracing/
use async_trait::async_trait;
use log::{error, info, LevelFilter};
use nvim_rs::{compat::tokio::Compat, create::tokio as create, Handler, Neovim, Value};
use simplelog::WriteLogger;
use std::{collections::HashMap, panic, sync::Arc};
use tokio::{io::Stdout, runtime, sync::RwLock};

mod collections;
mod nvim;

use nvim::iskeyword;

#[derive(Debug, Clone)]
struct NeovimHandler {
    iskeyword_map: Arc<RwLock<HashMap<u64, iskeyword::KeywordMatcher>>>,
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
            _ => (),
        }
    }
}

async fn run() {
    let (nvim, io_handler) = create::new_parent(NeovimHandler {
        iskeyword_map: Arc::new(RwLock::new(HashMap::new())),
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
