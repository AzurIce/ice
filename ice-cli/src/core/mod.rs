use std::{sync::{mpsc, Arc, Mutex}, thread, io::stdin, path::{Path, PathBuf}, fs::{self, DirEntry}};

use ice_core::utils::{time::get_cur_time_str, fs::copy_dir};
use log::{info, error, warn};

use crate::config::Config;

use self::server::Server;

pub mod server;
pub mod loader;
pub mod command;

pub enum Event {
    ServerDown,
    ServerLog(String),
    PlayerMessage { player: String, msg: String },
}

pub struct Core {
    pub config: Config,

    pub output_tx: mpsc::Sender<String>,  // Sender for stdout_loop
    pub command_tx: mpsc::Sender<String>, // Sender for command_hanle_loop
    pub event_tx: mpsc::Sender<Event>,

    pub running_server: Arc<Mutex<Option<Server>>>,
    pub pending_confirm: bool,
}

impl Core {
    pub fn run(config: Config) {
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
                    Event::PlayerMessage { player, msg } => {
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
            output_tx,
            command_tx,
            event_tx,
            running_server,
            pending_confirm: false
        };

        while let Ok(command) = command_rx.recv() {
            core.handle_command(command);
        }
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
                    *server = Some(Server::run(self.config.clone(), self.event_tx.clone()));
                }
            }
            "bksnap" => {
                if args.is_empty() || args[0] == "list" {
                    let snapshot_list = get_snapshot_list();

                    self.say("snapshots: ");
                    for (i, snapshot) in snapshot_list.into_iter().enumerate() {
                        self.say(format!("{i}: {snapshot:?}"))
                    }
                } else if args[0] == "make" {
                    while get_snapshot_list().len() >= 10 {
                        del_snapshot()
                    }
                    self.say("saving snapshot...");
                    make_snapshot();
                    self.say("saved snapshot")
                } else if args.len() == 2 && args[0] == "load" {
                    println!("bksnap load, not implemented yet")
                    // TODO: load snap backup
                }
            }
            "bkarch" => {
                if args.is_empty() || args[0] == "list" {
                    let archive_list = get_archive_list();

                    self.say("archives: ");
                    for (i, archive) in archive_list.into_iter().enumerate() {
                        self.say(format!("{i}: {archive:?}"))
                    }
                } else if args[0] == "make" {
                    let comment = args[1..].join(" ");
                    self.say("saving archive...");
                    make_archive(&comment);
                    self.say("saved archive")
                } else if args.len() == 2 && args[0] == "load" {
                    println!("bkarch load, not implemented yet")
                    // TODO: load arch backup
                }
            }
            _ => {
                println!("unknown command")
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


// Backup related
pub fn del_snapshot() {
    info!("deleting snapshot");
    let snapshot_dir = Path::new("./backups").join("snapshots");
    if let Err(err) = fs::create_dir_all(&snapshot_dir) {
        println!("failed to create all dir: {err}");
        return;
    }

    if let Ok(entries) = fs::read_dir(snapshot_dir) {
        let mut entries: Vec<DirEntry> = entries.into_iter().map(|entry| entry.unwrap()).collect();

        entries.sort_by_key(|entry| entry.metadata().unwrap().created().unwrap());
        let entries = entries
            .into_iter()
            .map(|entry| entry.path())
            .collect::<Vec<PathBuf>>();
        if let Some(first) = entries.first() {
            println!("[del_snapshop]: Deleting {first:?}...");
            if let Err(err) = fs::remove_dir_all(first) {
                println!("Failed to remove dir: {err}")
            }
            println!("[del_snapshop]: Snapshop deleted");
        }
    }
}

pub fn make_snapshot() {
    let snapshot_dir = Path::new("./backups").join("snapshots");
    if let Err(err) = fs::create_dir_all(snapshot_dir) {
        error!("failed to create all dir: {err}");
        return;
    }

    let src_path = Path::new(&"./server/").join("world");
    if !src_path.exists() {
        warn!("skip world/, not exist");
        return;
    }

    let backup_name = get_cur_time_str();
    let dst_path = Path::new("./backups").join("snapshots").join(backup_name);
    info!("copying from {src_path:?} to {dst_path:?}...");
    if let Err(err) = copy_dir(&src_path, &dst_path) {
        error!("failed to copy: {err}")
    }
}

pub fn get_snapshot_list() -> Vec<PathBuf> {
    let snapshot_dir = Path::new("./backups").join("snapshots");
    if let Err(err) = fs::create_dir_all(&snapshot_dir) {
        error!("failed to create all dir: {err}");
        return Vec::new();
    }

    if let Ok(entries) = fs::read_dir(snapshot_dir) {
        let mut entries: Vec<DirEntry> = entries.into_iter().map(|entry| entry.unwrap()).collect();

        entries.sort_by_key(|entry| entry.metadata().unwrap().created().unwrap());
        entries
            .into_iter()
            .map(|entry| entry.path())
            .collect::<Vec<PathBuf>>()
    } else {
        Vec::new()
    }
    // snapshot_list.sort_by_key(|snapshot|snapshot.metadata.created().unwrap());
    // snapshot_list
}

pub fn get_archive_list() -> Vec<PathBuf> {
    let dir = Path::new("./backups").join("archives");
    if let Err(err) = fs::create_dir_all(&dir) {
        error!("failed to create all dir: {err}");
        return Vec::new();
    }

    if let Ok(entries) = fs::read_dir(dir) {
        let mut entries: Vec<DirEntry> = entries.into_iter().map(|entry| entry.unwrap()).collect();

        entries.sort_by_key(|entry| entry.metadata().unwrap().created().unwrap());
        entries
            .into_iter()
            .map(|entry| entry.path())
            .collect::<Vec<PathBuf>>()
    } else {
        Vec::new()
    }
}

pub fn make_archive(name: &str) {
    let dir = Path::new("./backups").join("archives");
    if let Err(err) = fs::create_dir_all(&dir) {
        error!("failed to create all dir: {err}");
        return;
    }

    let src_path = Path::new(&"./server/").join("world");
    if !src_path.exists() {
        warn!("skip world/, not exist");
        return;
    }

    let backup_name = format!("{} {}", get_cur_time_str(), name);
    let dst_path = dir.join(backup_name);
    info!("copying from {src_path:?} to {dst_path:?}...");
    if let Err(err) = copy_dir(&src_path, &dst_path) {
        error!("failed to copy: {err}")
    }
}
