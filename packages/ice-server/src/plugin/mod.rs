pub mod rhai_plugin;
pub mod scoreboard;

pub use rhai_plugin::RhaiPlugin;

#[allow(unused)]
use std::time::Instant;

use std::any::Any;

use crate::{server::Server, Event};

#[allow(unused)]
pub trait Plugin: Any + Send + Sync {
    fn id(&self) -> &str;
    /// Called when server log, which is when something is printed to stdout
    fn on_server_log(&mut self, content: String) {}
    /// Called when the server is started up, which is when [`crate::server::regex::DONE`] is matched
    fn on_server_done(&mut self) {}
    /// Called when a player sends a message, which is when [`crate::server::regex::PLAYER`] is matched
    fn on_player_message(&mut self, player: String, msg: String) {}
    /// Called when the plugin is loaded
    fn on_load(&mut self) {}

    fn handle_event(&mut self, server: Server, event: Event) {
        match event {
            Event::ServerLog(content) => self.on_server_log(content),
            Event::ServerDone => self.on_server_done(),
            Event::PlayerMessage { player, msg } => self.on_player_message(player, msg),
            _ => (),
        }
    }
    // fn init(running_server: Arc<Mutex<Option<MinecraftServer>>>) -> impl Future<Output = Self>
    // where
    //     Self: Sized;
}
