use std::{
    collections::HashMap,
    fs,
    io::stdin,
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use crate::config::Config;
use command::{bkarch::BkArch, bksnap::BkSnap, Command};
use log::{error, info, warn};

use plugin::{scoreboard::ScoreBoard, Plugin};
use regex::Regex;
use server::Server;

pub mod command;
pub mod config;
pub mod plugin;
pub mod server;

pub enum Event {
    ServerDown,
    ServerDone,
    ServerLog(String),
    PlayerMessage { player: String, msg: String },
}

pub struct Core {
    pub config: Config,
    pub server_dir: PathBuf,
    commands: HashMap<String, Arc<Mutex<Box<dyn Command + Send + Sync>>>>,
    // plugins: Arc<Vec<Arc<Mutex<Box<dyn Plugin + Send>>>>>,
    pub output_tx: tokio::sync::mpsc::UnboundedSender<String>, // Sender for stdout_loop
    pub command_tx: mpsc::Sender<String>,                      // Sender for command_hanle_loop
    pub event_tx: tokio::sync::mpsc::UnboundedSender<Event>,

    pub running_server: Arc<Mutex<Option<Server>>>,
}

impl Core {
    pub async fn run<P: AsRef<Path>>(config: Config, server_dir: P) {
        let server_dir = server_dir.as_ref().to_owned();

        // Output
        let running_server = Arc::new(Mutex::new(None::<Server>));
        let (output_tx, mut output_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        tokio::spawn(async move {
            while let Some(buf) = output_rx.recv().await {
                println!("{buf}")
            }
        });

        // ? commands and plugins
        let mut commands = HashMap::<String, Arc<Mutex<Box<dyn Command + Send + Sync>>>>::new();
        let cmds: Vec<Box<dyn Command + Send + Sync>> =
            vec![Box::<BkSnap>::default(), Box::<BkArch>::default()];
        for cmd in cmds {
            commands.insert(cmd.cmd(), Arc::new(Mutex::new(cmd)));
        }

        let mut plugins: Vec<Box<dyn Plugin + Send>> =
            vec![Box::new(ScoreBoard::init(running_server.clone()).await)];
        // let plugins = Arc::new(plugins);

        let (command_tx, command_rx) = mpsc::channel::<String>();

        // Thread to forward inputs to server stdin or command thread
        let _running_server = running_server.clone();
        let _command_tx = command_tx.clone();
        thread::spawn(move || {
            let mut buf = String::new();
            while let Ok(_size) = stdin().read_line(&mut buf) {
                if buf.starts_with("#") {
                    _command_tx
                        .send(buf.clone())
                        .expect("failed to send to command_tx");
                } else {
                    let mut _running_server = _running_server.lock().unwrap();
                    if let Some(server) = _running_server.as_mut() {
                        server.writeln(&buf)
                    }
                }
                buf.clear();
            }
        });

        // Thread to handle server events
        let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<Event>();
        let _running_server = running_server.clone();
        let _command_tx = command_tx.clone();
        // let _plugins = plugins.clone();
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                match event {
                    Event::ServerDown => {
                        *_running_server.lock().unwrap() = None;
                    }
                    Event::ServerLog(msg) => {
                        println!("{msg}");

                        for plugin in &mut plugins {
                            plugin.on_server_log(msg.clone());
                        }
                    }
                    Event::PlayerMessage { player: _, msg } => {
                        if msg.starts_with("#") {
                            _command_tx
                                .send(msg.clone())
                                .expect("failed to send to command_tx");
                        }
                    }
                    Event::ServerDone => {
                        let mut server = _running_server.lock().unwrap();
                        for plugin in &mut plugins {
                            plugin.on_server_done(server.as_mut());
                        }
                    }
                }
            }
        });

        let mut core = Core {
            config,
            server_dir,
            commands,
            // plugins,
            output_tx,
            command_tx,
            event_tx,
            running_server,
        };

        while let Ok(command) = command_rx.recv() {
            core.handle_command(command);
        }
    }

    fn update_properties(&self) {
        info!("checking properties...");
        let path = self.server_dir.join("server.properties");
        if !path.exists() {
            warn!("server.properties not found, cannot patch, skipping...");
            return;
        }
        info!("patching properties...");
        let mut buf = fs::read_to_string(&path).expect("failed to read server.properties");

        for (key, value) in &self.config.properties {
            info!("setting property [{}] to [{}]", key, value);
            let regex = Regex::new(format!(r"{}=([^#\n\r]*)", key).as_str()).unwrap();
            buf = regex
                .replace(&buf, format!("{}={}", key, value))
                .to_string();
        }
        fs::write(path, buf.as_bytes()).expect("failed to write server.properties: {:?}");
    }

    fn handle_command(&mut self, command: String) {
        let command = command.replace("\r\n", "\n");
        let command = command.strip_prefix('#').unwrap();
        let command = command.strip_suffix('\n').unwrap_or(command);

        let split: Vec<&str> = command.split(' ').collect();
        let command = split[0];
        let args = &split[1..];

        info!("command: {} {:?}", command, args);

        match command {
            "start" => self.start_server(),
            _ => {
                let cmd = self.commands.get(command).cloned();
                if let Some(cmd) = cmd {
                    let mut cmd = cmd.lock().unwrap();
                    cmd.perform(self, args.iter().map(|s| s.to_string()).collect())
                } else {
                    println!("unknown command")
                }
            }
        }
    }

    pub fn start_server(&mut self) {
        let mut server = self.running_server.lock().unwrap();
        if server.is_some() {
            error!("server is already running");
        } else {
            self.update_properties();
            *server = Some(Server::run(self.config.clone(), self.event_tx.clone()));
        }
    }

    pub fn stop_server(&mut self) {
        if let Some(server) = self.running_server.lock().unwrap().as_mut() {
            server.writeln("stop");
        } else {
            info!("no running server");
        }
    }

    pub fn wait_till_stop(&self) {
        loop {
            if self.running_server.lock().unwrap().is_none() {
                break;
            }
            thread::sleep(Duration::from_secs_f32(0.2));
        }
    }

    pub fn say<S: AsRef<str>>(&mut self, content: S) {
        let content = content.as_ref();
        println!("{content}");
        if let Some(server) = self.running_server.lock().unwrap().as_mut() {
            server.writeln(format!("say {}", content).as_str());
        }
    }
}
