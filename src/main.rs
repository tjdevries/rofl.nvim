// Erik recommends: https://tracing.rs/tracing/
use async_trait::async_trait;
use log::{error, info, LevelFilter};
use nvim_rs::{compat::tokio::Compat, create::tokio as create, Handler, Neovim, Value};
use simplelog::WriteLogger;
use std::{fs::File, panic};
use tokio::{io::Stdout, runtime};

#[derive(Debug, Clone)]
struct NeovimHandler {}

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

    async fn handle_notify(&self, name: String, args: Vec<Value>, neovim: Neovim<Self::Writer>) {
        match name.as_ref() {
            "complete" => {}
            "v_char" => {}
            "insert_leave" => {}
            _ => (),
        }
    }
}

async fn run() {
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

    let (nvim, io_handler) = create::new_parent(NeovimHandler {}).await;
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

fn main() {
    let mut runtime = runtime::Builder::new()
        .threaded_scheduler()
        .build()
        .expect("Failed to build runtime");
    runtime.block_on(run())
}
