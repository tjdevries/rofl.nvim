// Erik recommends: https://tracing.rs/tracing/
mod entry;
mod score;
mod source;

use std::{cell::RefCell, fs::File, panic, sync::Arc, time::Duration};

use log::{debug, error, info, LevelFilter};

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
    sync::{mpsc::channel, Mutex, RwLock, RwLockReadGuard},
    time::Instant,
};

pub use entry::Entry;
pub use score::Score;
pub use source::{SharedSource, Source};

const CHANNEL_SIZE: usize = 500;

#[derive(Debug, Clone)]
struct Completor {
    v_char: Option<char>,
    user_match: Arc<RwLock<String>>,
    sources: Vec<SharedSource>,
    instant: Instant,
}

impl Completor {
    fn new() -> Completor {
        Completor {
            v_char: None,
            user_match: Arc::new(RwLock::new(String::new())),
            sources: Vec::new(),
            instant: Instant::now(),
        }
    }

    fn set_v_char(&mut self, c: Value) {
        let s: String = match c {
            Value::String(utf_s) => utf_s.into_str().expect("Couldn't convert to rust String"),
            _ => panic!("The value must be a string"),
        };
        let mut chars = s.chars();
        let maybe_c = chars.next().expect("String is empty");

        if let Some(c) = chars.next() {
            panic!("String is not only one char");
        }

        debug!("Setting v_char to {}", maybe_c);

        self.v_char = Some(maybe_c);
    }

    async fn update_user_match(&mut self) {
        let mut user_match = self.user_match.write().await;
        if let Some(' ') = self.v_char {
            user_match.clear();
        } else if let Some(c) = self.v_char {
            user_match.push(c)
        }
        debug!("the user match is now: {}", user_match);
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

        info!("completing");
        let nvim = shared_nvim.read().await;

        let (sender, mut receiver) = channel(CHANNEL_SIZE);

        // let mode = nvim.get_mode().await?.swap_remove(0).1;
        // let mode = mode.as_str().unwrap();
        // debug!("mode: {:?}", mode);
        // if mode != "i" || mode != "ic" {
        //     return Ok(());
        // }

        let mut futs = Vec::with_capacity(self.sources.len());

        let user_match = self.user_match.read().await;
        for source in &self.sources {
            let shared_nvim = shared_nvim.clone();
            let source = source.clone();
            let sender = sender.clone();
            let user_match = user_match.clone();
            let handle = tokio::spawn(async move {
                source
                    .lock()
                    .await
                    .get(shared_nvim, sender, &*user_match.clone())
                    .await
            });
            futs.push(handle);
        }
        drop(sender); // all the sources have their senders, we don't need it anymore

        join_all(futs)
            .await
            .into_iter()
            .map(|j| j.unwrap())
            .for_each(|_| ());

        let mut entries = Vec::new();
        while let Some(entry) = receiver.recv().await {
            info!("Got entry: {:?}", entry);
            entries.push(entry);
        }
        entries.sort_unstable_by(|e1, e2| e1.score.cmp(&e2.score));

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

        Ok(Value::from(true))
    }

    async fn handle_notify(
        &self,
        name: String,
        mut args: Vec<Value>,
        neovim: Neovim<Self::Writer>,
    ) {
        let nvim = SharedNvim::new(neovim);
        debug!("Notification: {}, {:?}", name, args);

        match name.as_ref() {
            "complete" => {
                self.completor
                    .lock()
                    .await
                    .complete(nvim)
                    .await
                    .expect("Failed to complete");
            }
            "v_char" => {
                let mut completor = self.completor.lock().await;
                completor.set_v_char(args.remove(0));
                drop(args);
                completor.update_user_match();
            }
            "insert_leave" => {
                let completor = self.completor.lock().await;
                info!("Clearing user match");
                completor.user_match.write().await.clear();
            }
            _ => (),
        }
    }
}

#[tokio::main]
async fn main() {
    WriteLogger::init(
        LevelFilter::Debug,
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
