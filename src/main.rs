// Erik recommends: https://tracing.rs/tracing/

use std::fs::File;

use async_std::{self, io::Stdout};
use log::{info, LevelFilter};

use async_trait::async_trait;
use nvim_rs::{create::async_std as create, Handler, Neovim, Value};
use simplelog::WriteLogger;

#[derive(Clone)]
// struct NeovimHandler(Arc<Mutex<Posis>>);
struct NeovimHandler {}

#[async_trait]
impl Handler for NeovimHandler {
    type Writer = Stdout;

    async fn handle_request(
        &self,
        name: String,
        args: Vec<Value>,
        neovim: Neovim<Self::Writer>,
    ) -> Result<Value, Value> {
        info!("Request: {}, {:?}", name, args);

        match name.as_ref() {
            "first" => {
                info!("Succesfully handled first");
                Ok(Value::from("FIRST"))
            }
            "complete" => {
                let buf = neovim
                    .get_current_buf()
                    .await
                    .expect("Always has one buffer");

                let lines = buf
                    .get_lines(0, -1, false)
                    .await
                    .expect("Always gets da line");

                Ok(Value::from(lines[0].as_str()))
            }
            "_test" => Ok(Value::from(true)),
            _ => Err(nvim_rs::Value::from("Not implemented")),
        }
    }

    async fn handle_notify(&self, name: String, args: Vec<Value>, _neovim: Neovim<Self::Writer>) {
        info!("Notification: {}, {:?}", name, args);

        match name.as_ref() {
            "PogChamp" => {
                info!("You, we got dat PogChamp");
            }
            _ => (),
        }
    }
}

#[async_std::main]
async fn main() {
    WriteLogger::init(
        LevelFilter::Info,
        simplelog::Config::default(),
        File::create("/home/brian/rofl.log").expect("Failed to create file"),
    )
    .expect("Failed to start logger");

    info!("Starting running the things");
    let handler: NeovimHandler = NeovimHandler {};

    let (nvim, io_handler) = create::new_parent(handler).await;
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
