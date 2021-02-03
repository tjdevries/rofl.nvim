mod entry;
mod nvim;
mod score;
mod source;

use std::{panic, sync::Arc, time::Duration};

use log::{debug, error, info, trace, LevelFilter};

use anyhow::Result;
use async_trait::async_trait;
use futures::future::AbortHandle;
use futures::{future::abortable, future::join_all};
use nvim_rs::{
    call_args, compat::tokio::Compat, create::tokio as create, rpc::model::IntoVal, Handler,
    Neovim, Value,
};
use simplelog::WriteLogger;
use tokio::{
    io::Stdout,
    runtime,
    sync::{Mutex, RwLock},
    task,
    time::Instant,
};

pub use entry::Entry;
pub use score::Score;
pub use source::{SharedSource, Source};

type SharedNvim = Arc<Neovim<Compat<Stdout>>>;

#[derive(Debug, Clone)]
struct Completor {
    v_char: Option<char>,
    user_match: Arc<RwLock<String>>,
    sources: Vec<SharedSource>,
    instant: Instant,
    complete_fut: Option<AbortHandle>,
}

impl Completor {
    fn new() -> Completor {
        Completor {
            v_char: None,
            user_match: Arc::new(RwLock::new(String::new())),
            sources: Vec::new(),
            instant: Instant::now(),
            complete_fut: None,
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

    async fn complete(&mut self, nvim: SharedNvim) -> Result<()> {
        // if self.quicker_than(Duration::from_millis(50)) {
        //     return Ok(());
        // }

        // let mode = nvim.get_mode().await?.swap_remove(0).1;
        // let mode = mode.as_str().unwrap();
        // debug!("mode: {:?}", mode);
        // if mode != "i" || mode != "ic" {
        //     return Ok(());
        // }

        let mut futs = Vec::with_capacity(self.sources.len());

        for source in &self.sources {
            let nvim = nvim.clone();
            let source = source.clone();
            let user_match = self.user_match.clone();

            let handle = task::spawn(async move {
                let mut source = source.lock().await;
                source.get(nvim, &user_match.read().await).await
            });
            futs.push(handle);
        }

        let user_match = self.user_match.read().await;
        let mut entries: Vec<Entry> = join_all(futs)
            .await
            .into_iter()
            .map(|res| res.expect("Failed to join"))
            .flatten()
            .filter_map(|e| e.score(&user_match))
            .collect();

        entries.sort_unstable_by(|e1, e2| e1.score.cmp(&e2.score));

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
        trace!("Notification: {}, {:?}", name, args);

        match name.as_ref() {
            "complete" => {
                if let Some(previous_complete) = self.completor.lock().await.complete_fut.take() {
                    previous_complete.abort();
                }

                let completor = self.completor();
                let fut = task::spawn(async move {
                    let mut completor = completor.lock().await;
                    completor.complete(nvim).await.expect("Failed to complete");
                });

                let (_fut, handle) = abortable(fut);
                self.completor.lock().await.complete_fut.replace(handle);
            }
            "v_char" => {
                let completor = self.completor();
                task::spawn(async move {
                    let mut completor_handle = completor.lock().await;
                    completor_handle.set_v_char(args.remove(0));
                    drop(args);
                    completor_handle.update_user_match().await;
                });
            }
            "insert_leave" => {
                let completor = self.completor();
                task::spawn(async move {
                    let completor = completor.lock().await;
                    info!("Clearing user match");
                    completor.user_match.write().await.clear();
                });
            }
            _ => (),
        }
    }
}

impl NeovimHandler {
    fn completor(&self) -> Arc<Mutex<Completor>> {
        self.completor.clone()
    }
}

async fn run() {
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
            trace!("OK Result: {:?}", res);
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
