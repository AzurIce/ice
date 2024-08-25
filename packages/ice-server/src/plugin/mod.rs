pub mod scoreboard;

#[allow(unused)]
use std::time::Instant;

use std::path::PathBuf;

use rhai::{Engine, EvalAltResult, Scope, AST};

use crate::server::Server;

#[allow(unused)]
pub trait Plugin {
    fn id(&self) -> &str;
    fn on_server_log(&mut self, server: Server, content: String) {}
    fn on_server_done(&mut self, server: Server) {}
    fn on_load(&mut self, server: Server) {}
    // fn init(running_server: Arc<Mutex<Option<MinecraftServer>>>) -> impl Future<Output = Self>
    // where
    //     Self: Sized;

    // Useless for Rust Plugin
    fn call_fn(&mut self, fn_name: String, server: Server) {}
}

pub struct RhaiPlugin<'a> {
    id: String,
    engine: Engine,
    scope: Scope<'a>,
    ast: AST,
}

impl<'a> RhaiPlugin<'a> {
    /*
        engine initializing cost: 8.72425ms
        ast compile cost: 1.621958ms
        first eval and id cost: 457.25µs
        register type and fn cost: 236.625µs
    */
    pub fn from_file(path: PathBuf) -> Self {
        // let t = Instant::now();
        let mut engine = Engine::new();
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
            .register_fn("delay_call", Server::delay_call);
        // println!("register type and fn cost: {:?}", t.elapsed());

        Self {
            id,
            engine,
            scope,
            ast,
        }
    }
}

impl<'a> Plugin for RhaiPlugin<'a> {
    fn id(&self) -> &str {
        &self.id
    }

    fn on_load(&mut self, server: Server) {
        let res = self
            .engine
            .call_fn::<()>(&mut self.scope, &self.ast, "on_load", (server,));
        if let Err(err) = res {
            if !matches!(*err, EvalAltResult::ErrorFunctionNotFound(_, _)) {
                println!("{err}")
            }
        }
    }

    fn on_server_log(&mut self, server: Server, content: String) {
        let res = self.engine.call_fn::<()>(
            &mut self.scope,
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
                .call_fn::<()>(&mut self.scope, &self.ast, "on_server_done", (server,));
        if let Err(err) = res {
            if !matches!(*err, EvalAltResult::ErrorFunctionNotFound(_, _)) {
                println!("{err}")
            }
        }
    }

    fn call_fn(&mut self, fn_name: String, server: Server) {
        let res = self
            .engine
            .call_fn::<()>(&mut self.scope, &self.ast, &fn_name, (server,));
        if let Err(err) = res {
            if !matches!(*err, EvalAltResult::ErrorFunctionNotFound(_, _)) {
                println!("{err}")
            }
        }
    }
}
