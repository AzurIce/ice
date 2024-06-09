pub mod server;

use std::{
    collections::HashMap,
    io::stdin,
    sync::{mpsc, Arc, Mutex},
    thread,
};

use log::info;

use crate::{config::BishConfig, core::server::Server, utils::regex::forward_regex};

use self::server::run;

pub struct Core {
    pub config: BishConfig,
    pub servers: Arc<Mutex<HashMap<String, Arc<Mutex<Server>>>>>,
    output_tx: mpsc::Sender<String>,
}

impl Core {
    pub fn init(config: BishConfig) -> Self {
        let mut servers = HashMap::<String, Arc<Mutex<Server>>>::new();
        for (name, config) in &config.servers {
            servers.insert(name.clone(), Server::init(name.clone(), config.clone()));
        }
        let servers = Arc::new(Mutex::new(servers));
        let (output_tx, output_rx) = mpsc::channel::<String>();

        // 输出处理线程
        // 统一处理来自所有 server 的输出
        thread::spawn(move || {
            while let Ok(buf) = output_rx.recv() {
                println!("{buf}")
            }
        });

        // 终端输入处理
        // 从终端接受输入，识别转发正则，转发到对应服务器的 input_tx
        // 或全部转发
        let _servers = servers.clone();
        thread::spawn(move || {
            let mut buf = String::new();
            while let Ok(_size) = stdin().read_line(&mut buf) {
                // 正则捕获目标服务器，转发至对应服务器。
                // 或全部转发
                if let Some(cap) = forward_regex().captures(&buf) {
                    let line = cap.get(1).unwrap().as_str();
                    let server_name = cap.get(2).unwrap().as_str();
                    if let Some(server) = _servers.lock().unwrap().get(server_name) {
                        let mut server = server.lock().unwrap();
                        server.writeln(line)
                    }
                } else {
                    for (_server_name, server) in _servers.lock().unwrap().iter() {
                        let mut server = server.lock().unwrap();
                        server.writeln(&buf)
                    }
                }
                buf.clear();
            }
        });

        Self {
            config,
            servers,
            output_tx,
        }
    }

    pub fn run_server(&self, name: String) -> Result<(), String> {
        info!("[core/run_server]: {}", name);
        if let Some(server) = self.servers.lock().unwrap().get(&name) {
            run(server.clone(), self.output_tx.clone())
        } else {
            Err("cannot find server".to_string())
        }
    }

    pub fn stop_server(&self, name: String) {
        if let Some(server) = self.servers.lock().unwrap().get(&name) {
            server.lock().unwrap().writeln("stop");
        }
    }
}
