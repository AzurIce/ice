pub mod lib;
pub mod scoreboard;

#[allow(unused)]
use std::time::Instant;

use std::{any::Any, path::PathBuf};

use lib::engine_with_lib;
use rhai::{Engine, EvalAltResult, FuncArgs, Scope, AST};
use tracing::error;

use crate::{server::Server, Event};

#[allow(unused)]
pub trait Plugin: Any + Send + Sync {
    fn id(&self) -> &str;
    fn on_server_log(&mut self, server: Server, content: String) {}
    fn on_server_done(&mut self, server: Server) {}
    fn on_player_message(&mut self, server: Server, player: String, msg: String) {}
    fn on_load(&mut self, server: Server) {}

    fn handle_event(&mut self, server: Server, event: Event) {
        match event {
            Event::ServerLog(content) => self.on_server_log(server, content),
            Event::ServerDone => self.on_server_done(server),
            Event::PlayerMessage { player, msg } => self.on_player_message(server, player, msg),
            _ => (),
        }
    }
    // fn init(running_server: Arc<Mutex<Option<MinecraftServer>>>) -> impl Future<Output = Self>
    // where
    //     Self: Sized;
}

pub struct RhaiPlugin {
    id: String,
    engine: Engine,
    scope: Scope<'static>,
    ast: AST,
}

impl RhaiPlugin {
    /*
        engine initializing cost: 8.72425ms
        ast compile cost: 1.621958ms
        first eval and id cost: 457.25µs
        register type and fn cost: 236.625µs
    */
    pub fn from_file(path: PathBuf) -> Self {
        // let t = Instant::now();
        let mut engine = engine_with_lib();
        // println!("engine initializing cost: {:?}", t.elapsed());

        // let t = Instant::now();
        let ast = engine.compile_file(path).unwrap();
        // println!("ast compile cost: {:?}", t.elapsed());

        // let t = Instant::now();
        let mut scope = Scope::new();
        engine.eval_ast_with_scope::<()>(&mut scope, &ast).unwrap();
        let id = engine
            .call_fn::<String>(&mut scope, &ast, "id", ())
            .unwrap();
        // println!("first eval and id cost: {:?}", t.elapsed());

        // let t = Instant::now();
        engine
            .build_type::<Server>()
            .register_fn("start", Server::start)
            .register_fn("stop", Server::stop)
            .register_fn("delay_call", Server::delay_call)
            .register_fn("running", Server::running)
            .register_fn("say", Server::say::<String>)
            .register_fn("writeln", Server::writeln);
        // println!("register type and fn cost: {:?}", t.elapsed());

        Self {
            id,
            engine,
            scope,
            ast,
        }
    }

    /// Calls a function in the plugin, skip if not exist
    pub fn call_fn(&mut self, fn_name: impl AsRef<str>, args: impl FuncArgs) {
        let fn_name = fn_name.as_ref();

        let res = self
            .engine
            .call_fn::<()>(&mut self.scope, &self.ast, &fn_name, args);
        if let Err(err) = res {
            if let EvalAltResult::ErrorFunctionNotFound(name, _) = err.as_ref() {
                if name != fn_name {
                    error!("{err}")
                }
            } else {
                error!("{err}")
            }
        }
    }
}

impl Plugin for RhaiPlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn on_load(&mut self, server: Server) {
        self.call_fn("on_load", (server,));
    }

    fn on_server_log(&mut self, server: Server, content: String) {
        self.call_fn("on_server_log", (server, content));
    }

    fn on_server_done(&mut self, server: Server) {
        self.call_fn("on_server_done", (server,));
    }

    fn on_player_message(&mut self, server: Server, player: String, msg: String) {
        self.call_fn("on_player_message", (server, player, msg));
    }
}

#[cfg(test)]
mod test {
    use std::{env, process::Command};

    use pyo3::prelude::*;

    #[test]
    fn foo() {
        env::set_current_dir("plugins/py-here").unwrap();
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let module = PyModule::import_bound(py, "numpy").expect("failed to import requests");
            py.eval_bound("print('hello from pyo3')", None, None).unwrap();
        });
    }
}
