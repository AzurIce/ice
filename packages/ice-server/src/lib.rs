use std::{
    collections::HashMap,
    fs::{self},
    io::stdin,
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use crate::config::Config;
use command::{bkarch::BkArch, bksnap::BkSnap, Command};

use plugin::{scoreboard::ScoreBoard, Plugin, RhaiPlugin};
use server::Server;
use tracing::info;

pub mod command;
pub mod config;
pub mod plugin;
pub mod server;

macro_rules! include_plugin {
    ($filename:expr) => {
        (include_bytes!(concat!("../plugins/", $filename)), $filename)
    };
}

const BUILTIN_PLUGINS: &[(&[u8], &str)] = &[include_plugin!("here.rhai")];

pub enum Event {
    ServerDown,
    ServerDone,
    ServerLog(String),
    PlayerMessage {
        player: String,
        msg: String,
    },
    PluginDelayCall {
        delay_ms: u64,
        plugin_id: String,
        fn_name: String,
    },
}

pub struct Core {
    pub config: Config,
    pub server_dir: PathBuf,
    commands: HashMap<String, Arc<Mutex<Box<dyn Command + Send + Sync>>>>,

    // plugins: Arc<Vec<Arc<Mutex<Box<dyn Plugin + Send>>>>>,
    pub output_tx: tokio::sync::mpsc::UnboundedSender<String>, // Sender for stdout_loop
    pub command_tx: mpsc::Sender<String>,                      // Sender for command_hanle_loop
    pub event_tx: tokio::sync::mpsc::UnboundedSender<Event>,

    pub server: Server,
}

impl Core {
    pub async fn run<P: AsRef<Path>>(config: Config, root_dir: P) {
        let root_dir = root_dir.as_ref();

        let server_dir = root_dir.join("server");
        let plugins_dir = root_dir.join("plugins");

        let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<Event>();
        let server = Server::new(config.clone(), event_tx.clone());

        // Output
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

        // initialize builtin plugins
        fs::create_dir_all(&plugins_dir).unwrap();
        for (builtin_plugin, filename) in BUILTIN_PLUGINS {
            let path = plugins_dir.join(filename);
            // if !path.exists() {
            fs::write(path, builtin_plugin).unwrap();
            // }
        }

        // get plugins paths
        let rhai_plugins = fs::read_dir(&plugins_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .map(|s| s.to_str().unwrap())
                    .unwrap_or("")
                    == "rhai"
            })
            .map(|entry| entry.path())
            .collect::<Vec<PathBuf>>();

        info!("found {} plugins: {:?}", rhai_plugins.len(), rhai_plugins);

        info!("loading plugins...");
        let mut rhai_plugins = rhai_plugins
            .into_iter()
            .map(|path| {
                let t = Instant::now();
                let plugin = RhaiPlugin::from_file(path);
                info!("loaded {}, cost {:?}", plugin.id(), t.elapsed());
                plugin
            })
            .collect::<Vec<RhaiPlugin>>();
        info!("all plugin loaded");
        for plugin in &mut rhai_plugins {
            plugin.on_load(server.clone());
        }

        let mut plugins: Vec<Box<dyn Plugin + Send>> =
            vec![Box::new(ScoreBoard::init(server.clone()).await)];
        plugins.extend(
            rhai_plugins
                .into_iter()
                .map(|p| Box::new(p) as Box<dyn Plugin + Send>),
        );
        let plugins = Arc::new(Mutex::new(plugins));

        let (command_tx, command_rx) = mpsc::channel::<String>();

        // Thread to forward inputs to server stdin or command thread
        let mut _server = server.clone();
        let _command_tx = command_tx.clone();
        thread::spawn(move || {
            let mut buf = String::new();
            while let Ok(_size) = stdin().read_line(&mut buf) {
                if buf.starts_with("#") {
                    _command_tx
                        .send(buf.clone())
                        .expect("failed to send to command_tx");
                } else {
                    _server.writeln(buf.as_str());
                }
                buf.clear();
            }
        });

        // Thread to handle server events
        let _command_tx = command_tx.clone();
        let mut _server = server.clone();
        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                match event {
                    Event::PluginDelayCall {
                        delay_ms,
                        plugin_id,
                        fn_name,
                    } => {
                        info!(
                            "delay call {} {} {}, waiting...",
                            delay_ms, plugin_id, fn_name
                        );
                        let _plugins = plugins.clone();
                        let _server = _server.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                            info!("delay call {} {} {}", delay_ms, plugin_id, fn_name);
                            let mut plugins = _plugins.lock().unwrap();
                            let plugins: &mut Vec<Box<dyn Plugin + Send>> = plugins.as_mut();
                            plugins
                                .iter_mut()
                                .find(|p| p.id() == plugin_id)
                                .and_then(|p| Some(p.call_fn(fn_name, _server)));
                        });
                    }
                    Event::ServerDown => {
                        _server.handle_event(event);
                    }
                    Event::ServerLog(msg) => {
                        println!("{msg}");

                        let mut plugins = plugins.lock().unwrap();
                        let plugins: &mut Vec<Box<dyn Plugin + Send>> = plugins.as_mut();
                        for plugin in plugins {
                            plugin.on_server_log(_server.clone(), msg.clone());
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
                        let mut plugins = plugins.lock().unwrap();
                        let plugins: &mut Vec<Box<dyn Plugin + Send>> = plugins.as_mut();
                        for plugin in plugins {
                            plugin.on_server_done(_server.clone());
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
            server,
        };

        while let Ok(command) = command_rx.recv() {
            core.handle_command(command);
        }
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
        if !self.server.running() {
            self.server.start().unwrap();
        }
    }

    pub fn stop_server(&mut self) {
        if self.server.running() {
            self.server.stop().unwrap();
        }
    }

    pub fn say<S: AsRef<str>>(&mut self, content: S) {
        self.server.say(content)
    }
}
