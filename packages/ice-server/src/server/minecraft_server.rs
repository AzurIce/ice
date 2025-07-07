use std::{
    io::{self, BufRead, Write}, path::{Path}, process::{ChildStdin, Command, Stdio}, thread
};

use super::regex::{done_regex, player_regex};
use ice_util::minecraft::rtext::{build_component, Component};
use tracing::{error, info};

use crate::config::Config;

use crate::Event;

pub struct MinecraftServer {
    pub child_in: ChildStdin,
}

impl MinecraftServer {
    pub fn run(jar_dir: &Path, config: &Config, event_tx: smol::channel::Sender<Event>) -> Self {

        info!("Server::start");

        let mut command = Command::new("java");
        let mut args = config.jvm_options.split(' ').collect::<Vec<&str>>();
        args.retain(|s| !s.is_empty());
        args.extend(["-jar", config.loader.launch_filename_str(), "--nogui"]);
        info!("Running command: java {}", args.join(" "));

        command.current_dir(jar_dir);
        command.args(args);

        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to spawn");

        let child_in = child.stdin.take().expect("Failed to open child's stdin");

        let child_out = child.stdout.take().expect("Failed to open child's stdout");
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
                        let buf = buf.replace("\r\n", "\n");
                        let buf = buf.strip_suffix("\n").unwrap_or(&buf).to_string();
                        if let Some(cap) = player_regex().captures(&buf) {
                            let player = cap.get(1).unwrap().as_str().to_string();
                            let msg = cap.get(2).unwrap().as_str().to_string();
                            smol::block_on(event_tx.send(Event::PlayerMessage { player, msg }))
                                .expect("failed to send to event_tx");
                            // if content.starts_with('#') {
                            //     command_tx
                            //         .send(content.to_string())
                            //         .expect("failed to send to command_tx");
                            // }
                        } else if done_regex().is_match(&buf) {
                            smol::block_on(event_tx.send(Event::ServerDone)).unwrap();
                        }
                        smol::block_on(event_tx
                            .send(Event::ServerLog(buf.clone())))
                            .expect("Failed to send to event_tx");
                        // println!("{buf}");
                    }
                }
            }
            info!("server end");
            child.wait().expect("failed to wait");
            smol::block_on(event_tx
                .send(Event::ServerDown))
                .expect("failed to send to event_tx");
        });

        Self { child_in }
    }

    pub fn writeln(&mut self, line: &str) {
        let line = if line.ends_with('\n') {
            line.to_string()
        } else {
            format!("{line}\n")
        };

        self.child_in
            .write_all(line.as_bytes())
            .expect("failed to write to server");
    }

    pub fn say<S: AsRef<str>>(&mut self, content: S) {
        let content = content.as_ref();
        self.writeln(format!("say {}", content).as_str());
    }

    pub fn tellraw(&mut self, target: impl AsRef<str>, component: impl Into<Component>) {
        let target = target.as_ref();
        let component = component.into();
        let component = build_component(component);
        self.writeln(format!("tellraw {target} {component}").as_str());
    }
}
