mod backup;

use std::{
    collections::HashMap,
    fmt::Display,
    fs::{self, File},
    io::{self, BufRead, Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread,
};

use log::{error, info};
use regex::Regex;

use crate::{
    config::{ServerConfig, ModInfo},
    fabric::init_server_jar,
    utils::{path::split_parent_and_file, regex::player_regex},
};

pub struct Server {
    pub name: String,
    pub config: ServerConfig,
    input_tx: Option<mpsc::Sender<String>>,
    command_tx: mpsc::Sender<String>,
    pub running: bool,
}

pub fn run(server: Arc<Mutex<Server>>, global_output_tx: Sender<String>) -> Result<(), String> {
    {
        let server = server.lock().unwrap();
        if server.running {
            error!("the server is already running");
            return Err("the server is already running".to_string());
        }
    }

    let cloned_server = server.clone();
    let mut server = server.lock().unwrap();
    server.running = true;
    info!("starting server<{}>...", server.name);
    let mut child = server
        .command()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn");

    let child_in = child.stdin.take().expect("Failed to open child's stdin");
    let child_out = child.stdout.take().expect("Failed to open child's stdout");

    // 统一输入处理线程
    // 一切从终端、服务端输出识别到的玩家输入，都会通过 input_tx 输入到 channel 中
    // 然后在此统一处理，识别命令，作相关处理
    let (input_tx, input_rx) = mpsc::channel::<String>();
    server.input_tx = Some(input_tx);

    // consumes input_rx
    let _command_tx = server.command_tx.clone();
    thread::spawn(move || {
        let mut writer = io::BufWriter::new(child_in);
        while let Ok(input) = input_rx.recv() {
            if input.starts_with('#') {
                _command_tx
                    .send(input)
                    .expect("failed to send to command_tx");
            } else {
                writer.write_all(input.as_bytes()).expect("failed to write");
                writer.flush().expect("failed to flush");
            }
        }
        info!("exit thread_input_rx")
    });

    // 服务端 输出处理线程
    // 通过 global_tx 发送给主线程统一处理
    // let global_output_tx = global_output_tx();
    let _command_tx = server.command_tx.clone();
    let _server = cloned_server.clone();
    thread::spawn(move || {
        let mut reader = io::BufReader::new(child_out);
        loop {
            let mut buf = String::new();
            match reader.read_line(&mut buf) {
                Err(err) => {
                    // TODO: 为何初次运行会有一段是 stream did not contain valid UTF-8？
                    error!("{}", err)
                }
                Ok(size) => {
                    if size == 0 {
                        info!("thread_read_output: readed Ok(0)");
                        break;
                    }
                    if let Some(cap) = player_regex().captures(&buf) {
                        let _player_name = cap.get(1).unwrap().as_str();
                        let content = cap.get(2).unwrap().as_str();
                        if content.starts_with('#') {
                            _command_tx
                                .send(content.to_string())
                                .expect("failed to send to command_tx");
                        }
                    }
                    global_output_tx
                        .send(buf.clone())
                        .expect("Failed to send to global_tx");
                    // println!("{buf}");
                }
            }
        }
        info!("server end");
        child.wait().expect("failed to wait");
        info!("exit thread_read_output");

        // Drop 掉 input_tx 使得上面的线程可以退出循环
        let mut server = _server.lock().unwrap();
        server.input_tx = None;
        server.running = false;
    });
    Ok(())
}

impl Server {
    pub fn init_properties(&self) -> Result<(), String> {
        self.set_properties(&self.config.properties)
    }

    // pub fn init_mods(&self) {
    //     for (mod_slug, mod_info) in self.config.mods {
    //         match mod_info {
    //             ModInfo::Latest() 
    //         }
    //     }
    // }

    pub fn init(name: String, config: ServerConfig) -> Arc<Mutex<Self>> {
        info!("server initializing...");
        let (command_tx, command_rx) = mpsc::channel::<String>();

        let server = Self {
            name,
            config,
            input_tx: None,
            command_tx,
            running: false,
        };

        info!("server initializing(properties)...");
        if let Err(err) = server.init_properties() {
            error!("server initializing(properties): error occured: {:?}", err)
        }

        info!("server initializing(mods)...");
        // server.init_mods();
        


        let server = Arc::new(Mutex::new(server));

        // 命令处理线程
        // 会一直运行，直至自身被 drop 掉（即 command_tx）
        // consumes command_rx
        let _server = server.clone();
        thread::spawn(move || {
            // let server = _server;
            while let Ok(command_str) = command_rx.recv() {
                let command_str = command_str.replace("\r\n", "\n");
                let command_str = command_str.strip_prefix('#').unwrap();
                let command_str = command_str.strip_suffix('\n').unwrap_or(command_str);

                let split: Vec<&str> = command_str.split(' ').collect();
                let command = split[0];
                let args = &split[1..];

                let mut server = _server.lock().unwrap();
                println!("[server/{}]: command: {} {:?}", server.name, command, args);
                match command {
                    "bksnap" => {
                        if args.is_empty() || args[0] == "list" {
                            let snapshot_list = server.get_snapshot_list();
                            server.say("snapshots: ");
                            for (i, snapshot) in snapshot_list.into_iter().enumerate() {
                                server.say(format!("{i}: {snapshot:?}"))
                            }
                        } else if args[0] == "make" {
                            while server.get_snapshot_list().len() >= 10 {
                                server.del_snapshot()
                            }
                            server.say("saving snapshot...");
                            server.make_snapshot();
                            server.say("saved snapshot")
                        } else if args.len() == 2 && args[0] == "load" {
                            // TODO: load snap backup
                        }
                    }
                    "bkarch" => {
                        if args.is_empty() || args[0] == "list" {
                            println!("bkarch list, not implemented yet")
                            // TODO: show arch backup
                        } else if args[0] == "make" {
                            println!("bkarch make, not implemented yet")
                            // let comment = args[1..].join(" ");
                            // TODO: make arch backup
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
            println!("exit thread_command_rx")
        });
        server
    }

    pub fn command(&self) -> Command {
        // init jar
        let (dir, filename) = split_parent_and_file(
            init_server_jar(&self.config.dir, &self.config.version)
                .expect("failed to init server jar"),
        );
        // init server.properties
        info!("setting properties...");
        if let Err(err) = self.set_properties(&self.config.properties) {
            error!("failed to set properties: {:?}, skipped", err);
        }

        let mut command = Command::new("java");
        let mut args = vec!["-jar", &filename, "--nogui"];
        args.extend(self.config.jvm_options.split(' ').collect::<Vec<&str>>());

        println!("[Server/new]: command's dir and jar_file is {dir} and {filename}");
        command.current_dir(dir);
        command.args(args);
        command
    }

    pub fn set_properties(&self, properties: &HashMap<String, String>) -> Result<(), String> {
        let property_file = PathBuf::from(&self.config.dir).join("server.properties");
        let mut buf = fs::read_to_string(&property_file)
            .map_err(|err| format!("failed to read server.properties: {:?}", err))?;

        for (key, value) in properties {
            info!("setting property [{}] to [{}]", key, value);
            let regex = Regex::new(format!(r"{}=([^#\n\r]*)", key).as_str()).unwrap();
            buf = regex
                .replace(&buf, format!("{}={}", key, value))
                .to_string();
        }
        fs::write(property_file, buf.as_bytes())
            .map_err(|err| format!("failed to write server.properties: {:?}", err))?;
        Ok(())
    }

    // pub fn load_snapshot(&self, id: usize) {
    // TODO: load snapshot
    // self.writeln("stop")
    // }

    pub fn writeln(&mut self, line: &str) {
        if let Some(tx) = &self.input_tx {
            let line = if line.ends_with('\n') {
                line.to_string()
            } else {
                format!("{line}\n")
            };

            tx.send(line).expect("failed to send to server's input_tx");
        }
    }

    pub fn say<S: AsRef<str> + Display>(&mut self, content: S) {
        self.writeln(&format!("say {content}"))
    }
}
