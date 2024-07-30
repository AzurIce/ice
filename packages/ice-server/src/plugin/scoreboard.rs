use std::sync::{Arc, Mutex};

use regex::Regex;
use tracing::info;

use crate::{server::Server};

use super::Plugin;

pub enum Events {
    ChangeSidebarObjective(String),
    UpdatedObjectives(Vec<String>),
}

pub struct ScoreBoard {
    objectives_regex: Regex,
    objectives: Arc<Mutex<Vec<String>>>,
    event_tx: tokio::sync::mpsc::UnboundedSender<Events>,
}

impl Plugin for ScoreBoard {
    fn id(&self) -> String {
        "scoreboard".to_string()
    }

    // When read scoreboard objectives list output, update state
    fn on_server_log(&mut self, content: String) {
        // info!("on_server_log: {content}");
        if let Some(capture) = self.objectives_regex.captures(&content) {
            let re = Regex::new(r"\[([^\]]+)\]").unwrap();

            let objectives = re
                .captures_iter(&capture.get(1).unwrap().as_str())
                .map(|cap| cap.get(1).unwrap().as_str().to_string())
                .collect::<Vec<String>>();
            self.event_tx
                .send(Events::UpdatedObjectives(objectives))
                .unwrap();
        }
    }

    // When server is done, get once scoreboard
    fn on_server_done(&mut self, server: Option<&mut Server>) {
        // info!("on_server_done");
        if let Some(server) = server {
            server.writeln("scoreboard objectives list")
        }
    }

    async fn init(running_server: Arc<Mutex<Option<Server>>>) -> Self {
        // info!("initializing...");
        let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<Events>();
        let objectives = Arc::new(Mutex::new(Vec::new()));

        // Send a change event every 10s
        let _event_tx = event_tx.clone();
        let _objectives = objectives.clone();
        tokio::spawn(async move {
            let mut index = 0;
            loop {
                {
                    let _objectives = _objectives.lock().unwrap();
                    if let Some(objective) = _objectives.get(index).cloned() {
                        _event_tx
                            .send(Events::ChangeSidebarObjective(objective))
                            .unwrap();
                        index = (index + 1) % _objectives.len();
                    } else {
                        index = 0;
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            }
        });

        let server = running_server;
        tokio::spawn(async move {
            loop {
                if let Some(event) = event_rx.recv().await {
                    match event {
                        Events::ChangeSidebarObjective(objective) => {
                            info!("Changing scoreboard display objective to {objective}...");
                            let mut server = server.lock().unwrap();
                            if let Some(server) = server.as_mut() {
                                server.writeln(&format!(
                                    "scoreboard objectives setdisplay sidebar {}",
                                    objective
                                ))
                            }
                        }
                        Events::UpdatedObjectives(objs) => {
                            info!("Updated objectives: {objs:?}");
                            *objectives.lock().unwrap() = objs;
                        }
                    }
                }
            }
        });

        let objectives_regex = Regex::new(r"]: There are \d+ objective\(s\): (.*)").unwrap();
        Self {
            objectives_regex,
            objectives: Arc::new(Mutex::new(vec![])),
            event_tx,
        }
    }
}
