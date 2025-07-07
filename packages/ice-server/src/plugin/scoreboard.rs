use std::sync::{Arc, Mutex};

use regex::Regex;
use tracing::info;

use crate::server::Server;

use super::Plugin;

pub enum Events {
    ChangeSidebarObjective(String),
    UpdatedObjectives(Vec<String>),
}

pub struct ScoreBoard {
    objectives_regex: Regex,
    event_tx: smol::channel::Sender<Events>,
    server: Server,
}

impl ScoreBoard {
    pub async fn init(server: Server) -> Self {
        // info!("initializing...");
        let (event_tx, event_rx) = smol::channel::unbounded::<Events>();
        let objectives = Arc::new(Mutex::new(Vec::new()));

        // Send a change event every 10s
        let _event_tx = event_tx.clone();
        let _objectives = objectives.clone();
        smol::spawn(async move {
            let mut index = 0;
            loop {
                {
                    let _objectives = _objectives.lock().unwrap();
                    if let Some(objective) = _objectives.get(index).cloned() {
                        smol::block_on(_event_tx.send(Events::ChangeSidebarObjective(objective)))
                            .unwrap();
                        index = (index + 1) % _objectives.len();
                    } else {
                        index = 0;
                    }
                }
                smol::Timer::after(std::time::Duration::from_secs(10)).await;
            }
        })
        .detach();

        let _server = server.clone();
        smol::spawn(async move {
            loop {
                if let Ok(event) = event_rx.recv().await {
                    match event {
                        Events::ChangeSidebarObjective(objective) => {
                            info!("Changing scoreboard display objective to {objective}...");
                            if _server.running() {
                                _server.writeln(&format!(
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
        })
        .detach();

        let objectives_regex = Regex::new(r"]: There are \d+ objective\(s\): (.*)").unwrap();
        Self {
            objectives_regex,
            event_tx,
            server,
        }
    }
}

impl Plugin for ScoreBoard {
    fn id(&self) -> &str {
        "scoreboard"
    }

    // When read scoreboard objectives list output, update state
    fn on_server_log(&mut self, content: String) {
        // info!("on_server_log: {content}");
        if let Some(capture) = self.objectives_regex.captures(&content) {
            let re = Regex::new(r"\[([^\]]+)\]").unwrap();

            let objectives = re
                .captures_iter(capture.get(1).unwrap().as_str())
                .map(|cap| cap.get(1).unwrap().as_str().to_string())
                .collect::<Vec<String>>();
            smol::block_on(self.event_tx.send(Events::UpdatedObjectives(objectives))).unwrap();
        }
    }

    // When server is done, get once scoreboard
    fn on_server_done(&mut self) {
        // info!("on_server_done");
        self.server.writeln("scoreboard objectives list")
    }
}
