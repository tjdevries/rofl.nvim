mod buffer;
mod counter;

use std::{fmt, sync::Arc};

use async_trait::async_trait;
use dyn_clone::DynClone;
use nvim_rs::{compat::tokio::Compat, Neovim};
use tokio::{io::Stdout, sync::{Mutex, RwLock}};

use super::{Entry, Nvim, Score};

pub use counter::Counter;

#[async_trait]
pub trait Source: 'static + Sync + Send + DynClone + fmt::Debug {
    async fn get(&mut self, nvim: Nvim) -> Vec<Entry>;
}

dyn_clone::clone_trait_object!(Source);

pub type SharedSource = Arc<Mutex<Box<dyn Source>>>;
