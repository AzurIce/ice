pub mod scoreboard;

use std::{path::PathBuf, time::Instant};

use rhai::{Engine, EvalAltResult, Scope, AST};

use crate::server::Server;

#[allow(unused)]
pub trait Plugin {
    fn id(&self) -> String;
    fn on_server_log(&mut self, server: Server, content: String) {}
    fn on_server_done(&mut self, server: Server) {}
    // fn init(running_server: Arc<Mutex<Option<MinecraftServer>>>) -> impl Future<Output = Self>
    // where
    //     Self: Sized;
}

pub struct RhaiPlugin {
    engine: Engine,
    ast: AST,
}

impl RhaiPlugin {
    pub fn from_file(path: PathBuf) -> Self {
        let t = Instant::now();
        let mut engine = Engine::new();
        engine
            .build_type::<Server>()
            .register_fn("start", Server::start);
        println!("engine initializing cost: {:?}", t.elapsed());

        let t = Instant::now();
        let ast = engine.compile_file(path).unwrap();
        println!("ast compile cost: {:?}", t.elapsed());

        Self { engine, ast }
    }
}

impl Plugin for RhaiPlugin {
    fn id(&self) -> String {
        "unknown".to_string()
    }

    fn on_server_log(&mut self, server: Server, content: String) {
        let res = self.engine.call_fn::<()>(
            &mut Scope::new(),
            &self.ast,
            "on_server_log",
            (server, content),
        );
        if let Err(err) = res {
            if !matches!(*err, EvalAltResult::ErrorFunctionNotFound(_, _)) {
                println!("{err}")
            }
        }
    }

    fn on_server_done(&mut self, server: Server) {
        let res =
            self.engine
                .call_fn::<()>(&mut Scope::new(), &self.ast, "on_server_done", (server,));
        if let Err(err) = res {
            if !matches!(*err, EvalAltResult::ErrorFunctionNotFound(_, _)) {
                println!("{err}")
            }
        }
    }
}
