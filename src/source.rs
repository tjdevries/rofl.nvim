mod buffer;
mod counter;
mod r#static;

use std::{fmt, sync::Arc};

use async_trait::async_trait;
use dyn_clone::DynClone;
use nvim_rs::{compat::tokio::Compat, Neovim};
use tokio::{
    io::Stdout,
    sync::{mpsc::Sender, Mutex, RwLock},
};

use super::{Entry, Score, SharedNvim};

pub use counter::Counter;
pub use r#static::Static;

#[async_trait]
pub trait Source: 'static + Sync + Send + DynClone + fmt::Debug {
    async fn get(&mut self, nvim: SharedNvim, sender: Sender<Entry>, user_match: &str);
}

dyn_clone::clone_trait_object!(Source);

pub type SharedSource = Arc<Mutex<Box<dyn Source>>>;
