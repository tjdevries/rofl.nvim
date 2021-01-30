// Erik recommends: https://tracing.rs/tracing/
mod entry;
mod score;
mod source;

use std::{cell::RefCell, fs::File, sync::Arc};

use log::{info, LevelFilter};

use anyhow::Result;
use async_trait::async_trait;
use futures::future::join_all;
use futures_util::stream;
use nvim_rs::{
    call_args, compat::tokio::Compat, create::tokio as create, rpc::model::IntoVal, Handler,
    Neovim, Value,
};
use simplelog::WriteLogger;
use tokio::{
    io::Stdout,
    sync::{Mutex, RwLock},
    task,
};

pub use entry::Entry;
pub use score::Score;
pub use source::{SharedSource, Source};

#[derive(Debug, Clone, Default)]
struct Completor {
    v_char: Option<char>,
    sources: Vec<SharedSource>,
}

impl Completor {
    fn new() -> Completor {
        Completor {
            v_char: None,
            sources: Vec::new(),
        }
    }

    async fn complete(&mut self, nvim: Nvim) -> Result<()> {
        let nvim_h = nvim.read().await;
        let mut futs = Vec::with_capacity(self.sources.len());
        for source in &self.sources {
            let nvim = nvim.clone();
            let source = source.clone();
            let handle = tokio::spawn(async move { source.lock().await.get(nvim).await });
            futs.push(handle);
        }
        let entries: Vec<Entry> = join_all(futs)
            .await
            .into_iter()
            .map(|res| res.unwrap())
            .flatten()
            .collect();
        info!("Completing with these entries: {:?}", entries);
        let entries = Entry::serialize(entries);
        nvim_h.call("complete", call_args!(1, entries)).await?;
        nvim_h.command("echo 'hello'").await?;
        Ok(())
    }

    fn register_source<S: Source>(&mut self, source: S) {
        self.sources.push(Arc::new(Mutex::new(Box::new(source))))
    }
}

#[derive(Debug, Default, Clone)]
struct NeovimHandler {
    completor: Arc<Mutex<Completor>>, // we want it to block
}

type Nvim = Arc<RwLock<Neovim<Compat<Stdout>>>>;

#[async_trait]
impl Handler for NeovimHandler {
    type Writer = Compat<Stdout>;

    async fn handle_request(
        &self,
        name: String,
        args: Vec<Value>,
        neovim: Neovim<Compat<Stdout>>,
    ) -> Result<Value, Value> {
        info!("Request: {}, {:?}", name, args);

        match name.as_ref() {
            "first" => {
                info!("Succesfully handled first");
                Ok(Value::from("FIRST"))
            }
            "insert_char_pre" => Ok("".into_val()),
            "_test" => Ok(Value::from(true)),
            _ => Err(nvim_rs::Value::from("Not implemented")),
        }
    }

    async fn handle_notify(&self, name: String, args: Vec<Value>, neovim: Neovim<Self::Writer>) {
        info!("Notification: {}, {:?}", name, args);
        let nvim = Arc::new(RwLock::new(neovim));

        match name.as_ref() {
            "complete" => {
                self.completor.lock().await.complete(nvim).await;
            }
            _ => (),
        }
    }
}

#[tokio::main]
async fn main() {
    WriteLogger::init(
        LevelFilter::Info,
        simplelog::Config::default(),
        File::create("/home/brian/rofl.log").expect("Failed to create file"),
    )
    .expect("Failed to start logger");

    info!("Starting running the things");
    let mut completor = Completor::new();
    completor.register_source(source::Counter(0));
    let (nvim, io_handler) = create::new_parent(NeovimHandler {
        completor: Arc::new(Mutex::new(completor)),
        // ..Default::default()
    })
    .await;
    info!("Connected to parent...");

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
