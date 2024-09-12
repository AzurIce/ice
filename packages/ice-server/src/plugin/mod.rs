pub mod rhai_plugin;
#[deprecated(since = "0.1.0-alpha.30", note = "use rhai_plugin instead")]
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

    /// only for [`RhaiPlugin`]
    fn on_call_fn(&mut self, fn_name: String) {}

    fn handle_event(&mut self, server: Server, event: Event) {
        match event {
            Event::ServerLog(content) => self.on_server_log(content),
            Event::ServerDone => self.on_server_done(),
            Event::PlayerMessage { player, msg } => self.on_player_message(player, msg),
            Event::PluginCallFn { plugin_id, fn_name } => {
                if self.id() == plugin_id {
                    self.on_call_fn(fn_name);
                }
            }
            _ => (),
        }
    }
}
