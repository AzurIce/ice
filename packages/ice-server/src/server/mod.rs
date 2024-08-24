use std::sync::{Arc, Mutex};

use minecraft_server::MinecraftServer;
use rhai::{CustomType, TypeBuilder};
use tracing::error;

use crate::{config::Config, Event};

pub mod minecraft_server;
pub mod regex;

#[derive(Clone, CustomType)]
pub struct Server {
    config: Config,
    event_tx: tokio::sync::mpsc::UnboundedSender<Event>,
    minecraft_server: Arc<Mutex<Option<MinecraftServer>>>,
}

impl Server {
    pub fn new(config: Config, event_tx: tokio::sync::mpsc::UnboundedSender<Event>) -> Self {
        Self {
            config,
            event_tx,
            minecraft_server: Arc::new(Mutex::new(None)),
        }
    }

    pub fn running(&mut self) -> bool {
        self.minecraft_server.lock().unwrap().is_some()
    }

    pub fn start(&mut self) -> Result<(), String> {
        let mut server = self.minecraft_server.lock().unwrap();
        if server.is_some() {
            error!("server is already running");
            Err(format!("server is already running"))
        } else {
            *server = Some(MinecraftServer::run(
                self.config.clone(),
                self.event_tx.clone(),
            ));
            Ok(())
        }
    }

    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(server) = self.minecraft_server.lock().unwrap().as_mut() {
            server.writeln("stop");
            Ok(())
        } else {
            error!("no running server");
            Err(format!("no running server"))
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::ServerDown => {
                *self.minecraft_server.lock().unwrap() = None;
            }
            _ => (),
        }
    }

    pub fn writeln(&mut self, line: &str) {
        let mut server = self.minecraft_server.lock().unwrap();
        if let Some(server) = server.as_mut() {
            server.writeln(line)
        }
    }

    pub fn say<S: AsRef<str>>(&mut self, content: S) {
        let content = content.as_ref();
        println!("{content}");
        if let Some(server) = self.minecraft_server.lock().unwrap().as_mut() {
            server.writeln(format!("say {}", content).as_str());
        }
    }
}
