// Erik recommends: https://tracing.rs/tracing/
mod entry;
mod score;
mod source;

use std::{cell::RefCell, fs::File, panic, sync::Arc, time::Duration};

use log::{error, info, trace, LevelFilter};

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
    sync::{Mutex, RwLock, RwLockReadGuard},
    time::Instant,
};

pub use entry::Entry;
pub use score::Score;
pub use source::{SharedSource, Source};

#[derive(Debug, Clone)]
struct Completor {
    v_char: Option<char>,
    sources: Vec<SharedSource>,
    instant: Instant,
}

impl Completor {
    fn new() -> Completor {
        Completor {
            v_char: None,
            sources: Vec::new(),
            instant: Instant::now(),
        }
    }

    fn quicker_than(&mut self, duration: Duration) -> bool {
        let earlier = self.instant;
        let now = Instant::now();
        self.instant = now;
        now.duration_since(earlier) < duration
    }

    async fn complete(&mut self, shared_nvim: SharedNvim) -> Result<()> {
        // if self.quicker_than(Duration::from_millis(50)) {
        //     return Ok(());
        // }

        let nvim = shared_nvim.read().await;

        let mut futs = Vec::with_capacity(self.sources.len());
        for source in &self.sources {
            let shared_nvim = shared_nvim.clone();
            let source = source.clone();
            let handle = tokio::spawn(async move { source.lock().await.get(shared_nvim).await });
            futs.push(handle);
        }

        let entries: Vec<Entry> = join_all(futs)
            .await
            .into_iter()
            .map(|res| res.expect("Failed to join_all"))
            .flatten()
            .collect();

        info!("Completing with these entries: {:?}", entries);
        let entries = Entry::serialize(entries);
        nvim.call_function(
            "complete",
            call_args!(nvim.call_function("col", call_args!(".")).await?, entries),
        )
        .await?;
        Ok(())
    }

    fn register<S: Source>(&mut self, source: S) {
        self.sources.push(Arc::new(Mutex::new(Box::new(source))))
    }
}

#[derive(Debug, Clone)]
struct NeovimHandler {
    completor: Arc<Mutex<Completor>>,
}

#[derive(Clone)]
pub struct SharedNvim(Arc<RwLock<Neovim<Compat<Stdout>>>>);

impl SharedNvim {
    fn new(neovim: Neovim<Compat<Stdout>>) -> SharedNvim {
        SharedNvim(Arc::new(RwLock::new(neovim)))
    }

    async fn read(&self) -> RwLockReadGuard<'_, Neovim<Compat<Stdout>>> {
        self.0.read().await
    }
}

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
        let nvim = SharedNvim::new(neovim);
        trace!("Notification: {}, {:?}", name, args);

        match name.as_ref() {
            "complete" => {
                self.completor
                    .lock()
                    .await
                    .complete(nvim)
                    .await
                    .expect("Failed to complete");
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

    // we do not want to crash when panicking, instead log it
    panic::set_hook(Box::new(move |panic| {
        error!("----- Panic -----");
        error!("{}", panic);
    }));

    let mut completor = Completor::new();
    completor.register(source::Counter(0));
    completor.register(source::Static::new(&[
        "This is just a test".to_owned(),
        "This is another test from static source".to_owned(),
    ]));

    let (nvim, io_handler) = create::new_parent(NeovimHandler {
        completor: Arc::new(Mutex::new(completor)),
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
