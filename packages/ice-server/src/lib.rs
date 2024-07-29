use std::{
    collections::HashMap,
    fs,
    io::stdin,
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
    thread,
};

use crate::config::Config;
use command::{bkarch::BkArch, bksnap::BkSnap, Command};
use log::{error, info, warn};

use regex::Regex;
use server::Server;

pub mod command;
pub mod config;
pub mod server;

pub enum Event {
    ServerDown,
    ServerLog(String),
    PlayerMessage { player: String, msg: String },
}

pub struct Core {
    pub config: Config,
    pub server_dir: PathBuf,
    commands: HashMap<String, Arc<Mutex<Box<dyn Command + Send + Sync>>>>,

    pub output_tx: mpsc::Sender<String>, // Sender for stdout_loop
    pub command_tx: mpsc::Sender<String>, // Sender for command_hanle_loop
    pub event_tx: mpsc::Sender<Event>,

    pub running_server: Arc<Mutex<Option<Server>>>,
    pub pending_confirm: bool,
}

impl Core {
    pub fn run<P: AsRef<Path>>(config: Config, server_dir: P) {
        let server_dir = server_dir.as_ref().to_owned();

        let mut commands = HashMap::<String, Arc<Mutex<Box<dyn Command + Send + Sync>>>>::new();
        let cmds: Vec<Box<dyn Command + Send + Sync>> =
            vec![Box::<BkSnap>::default(), Box::<BkArch>::default()];
        for cmd in cmds {
            commands.insert(cmd.cmd(), Arc::new(Mutex::new(cmd)));
        }

        let running_server = Arc::new(Mutex::new(None::<Server>));
        let (output_tx, output_rx) = mpsc::channel::<String>();
        thread::spawn(move || {
            while let Ok(buf) = output_rx.recv() {
                println!("{buf}")
            }
        });
        let (command_tx, command_rx) = mpsc::channel::<String>();

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

        let (event_tx, event_rx) = mpsc::channel::<Event>();
        let _running_server = running_server.clone();
        let _command_tx = command_tx.clone();
        thread::spawn(move || {
            while let Ok(event) = event_rx.recv() {
                match event {
                    Event::ServerDown => {
                        *_running_server.lock().unwrap() = None;
                    }
                    Event::ServerLog(msg) => {
                        println!("{msg}")
                    }
                    Event::PlayerMessage { player: _, msg } => {
                        if msg.starts_with("#") {
                            _command_tx
                                .send(msg.clone())
                                .expect("failed to send to command_tx");
                        }
                    }
                }
            }
        });
        let mut core = Core {
            config,
            server_dir,
            commands,
            output_tx,
            command_tx,
            event_tx,
            running_server,
            pending_confirm: false,
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
        let server = self.running_server.clone();

        let command = command.replace("\r\n", "\n");
        let command = command.strip_prefix('#').unwrap();
        let command = command.strip_suffix('\n').unwrap_or(command);

        let split: Vec<&str> = command.split(' ').collect();
        let command = split[0];
        let args = &split[1..];

        info!("command: {} {:?}", command, args);

        match command {
            "start" => {
                info!("command start");
                let mut server = server.lock().unwrap();
                if server.is_some() {
                    error!("server is already running");
                } else {
                    self.update_properties();
                    *server = Some(Server::run(self.config.clone(), self.event_tx.clone()));
                }
            }
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

    pub fn say<S: AsRef<str>>(&mut self, content: S) {
        let content = content.as_ref();
        println!("{content}");
        if let Some(server) = self.running_server.lock().unwrap().as_mut() {
            server.writeln(format!("say {}", content).as_str());
        }
    }
}
