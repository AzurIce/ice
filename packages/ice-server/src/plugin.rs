pub mod scoreboard;

use std::{
    future::Future,
    sync::{Arc, Mutex},
};

use crate::server::Server;

pub trait Plugin {
    fn id(&self) -> String;
    fn on_server_log(&mut self, content: String) {}
    fn on_server_done(&mut self, server: Option<&mut Server>) {}
    fn init(running_server: Arc<Mutex<Option<Server>>>) -> impl Future<Output = Self>
    where
        Self: Sized;
}
