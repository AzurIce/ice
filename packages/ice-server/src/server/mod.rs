use std::{path::PathBuf, sync::{Arc, Mutex}};

use ::regex::Regex;
use ice_util::minecraft::rtext::{build_component, Component};
use minecraft_server::MinecraftServer;
use tracing::{error, info};

use crate::{
    config::{Config, PluginConfig},
    Event,
};

pub mod minecraft_server;
pub mod regex;

#[derive(Clone)]
pub struct Server {
    jar_dir: PathBuf,
    config: Config,
    event_tx: smol::channel::Sender<Event>,
    minecraft_server: Arc<Mutex<Option<MinecraftServer>>>,
    log_filters: Arc<Mutex<Vec<Regex>>>,
}

impl Server {
    pub fn new(jar_dir: PathBuf, config: Config, event_tx: smol::channel::Sender<Event>) -> Self {
        Self {
            jar_dir,
            config,
            event_tx,
            minecraft_server: Arc::new(Mutex::new(None)),
            log_filters: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn retain_log(&self, content: &str) -> bool {
        let res = self
            .log_filters
            .lock()
            .unwrap()
            .iter()
            .all(|filter| !filter.is_match(content));
        res
    }

    pub fn add_log_filter(&mut self, filter: Regex) {
        self.log_filters.lock().unwrap().push(filter);
    }

    pub fn running(&self) -> bool {
        self.minecraft_server.lock().unwrap().is_some()
    }

    /// Start the server
    pub fn start(&self) -> Result<(), String> {
        info!("[server]: start");
        let mut server = self.minecraft_server.lock().unwrap();
        if server.is_some() {
            error!("server is already running");
            Err(format!("server is already running"))
        } else {
            *server = Some(MinecraftServer::run(
                &self.jar_dir,
                &self.config,
                self.event_tx.clone(),
            ));
            Ok(())
        }
    }

    /// Call a function in the plugin after a delay
    pub fn delay_call(&self, delay_ms: i64, plugin_id: String, fn_name: String) {
        smol::block_on(self.event_tx.send(Event::PluginDelayCall {
            delay_ms: delay_ms as u64,
            plugin_id,
            fn_name,
        }))
        .unwrap();
    }

    /// Get the config of a plugin
    pub fn get_plugin_config(&self, plugin_id: String) -> Option<&PluginConfig> {
        println!("get_plugin_config");
        self.config.plugins.get(&plugin_id)
    }

    /// Stop the server (write `stop` to the stdin of the server)
    pub fn stop(&self) -> Result<(), String> {
        if let Some(server) = self.minecraft_server.lock().unwrap().as_mut() {
            server.writeln("stop");
            Ok(())
        } else {
            error!("no running server");
            Err(format!("no running server"))
        }
    }

    pub fn handle_event(&self, event: Event) {
        match event {
            Event::ServerDown => {
                *self.minecraft_server.lock().unwrap() = None;
            }
            _ => (),
        }
    }

    /// Write a line to the stdin of the server
    pub fn writeln(&self, line: &str) {
        let mut server = self.minecraft_server.lock().unwrap();
        if let Some(server) = server.as_mut() {
            server.writeln(line)
        }
    }

    /// Say contents (write `say <content>` to the stdin of the server)
    pub fn say<S: AsRef<str>>(&self, content: S) {
        let content = content.as_ref();
        println!("say {content}");
        if let Some(server) = self.minecraft_server.lock().unwrap().as_mut() {
            server.writeln(format!("say {}", content).as_str());
        }
    }

    /// Tellraw to a target (write `tellraw <target> <component>` to the stdin of the server)
    pub fn tellraw<T: Into<Component>>(&mut self, target: String, component: T) {
        let component = component.into();
        println!("tellraw {target} {}", build_component(component.clone()));
        if let Some(server) = self.minecraft_server.lock().unwrap().as_mut() {
            server.tellraw(target, component);
        }
    }
}
