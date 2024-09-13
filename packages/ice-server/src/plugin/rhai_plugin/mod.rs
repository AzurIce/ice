#![warn(missing_docs)]
use std::path::PathBuf;

use ice_util::minecraft::rtext::{Component, ComponentObject};
use minecraft_rtext::MinecraftRtextPackage;
use ::regex::Regex;
use rhai::{
    packages::Package,
    serde::{from_dynamic, to_dynamic},
    CallFnOptions, CustomType, Engine, EvalAltResult, FuncArgs, Scope, TypeBuilder, AST,
};
use rhai_fs::FilesystemPackage;
use tracing::error;

pub mod minecraft_rtext;
mod regex;

pub(crate) fn engine_with_lib() -> Engine {
    let mut engine = Engine::new();

    let pkg = MinecraftRtextPackage::new();
    pkg.register_into_engine(&mut engine);

    engine.register_static_module("regex", rhai::exported_module!(regex::module).into());
    let package = FilesystemPackage::new();
    package.register_into_engine(&mut engine);
    engine
}

use super::Plugin;

/// An Ice plugin that is written in rhai
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
    /// Create a [`RhaiPlugin`] for a specific [`Server`] from a `.rhai` file
    pub fn from_file(server: crate::server::Server, path: PathBuf) -> Self {
        let server = Server { inner: server };

        // let t = Instant::now();
        let mut engine = engine_with_lib();
        // println!("engine initializing cost: {:?}", t.elapsed());

        // let t = Instant::now();
        // ? Compile the plugin
        let ast = engine.compile_file(path).unwrap();
        // println!("ast compile cost: {:?}", t.elapsed());

        // let t = Instant::now();
        let mut scope = Scope::new();
        // ? get id()
        let id = engine
            .call_fn_with_options::<String>(
                CallFnOptions::new().eval_ast(false),
                &mut scope,
                &ast,
                "id",
                (),
            )
            .unwrap();
        // println!("first eval and id cost: {:?}", t.elapsed());

        // let t = Instant::now();
        // ? Register apis
        engine.build_type::<Server>();
        let _server = server.clone();
        engine.register_fn("server", move || _server.clone());
        let _server = server.clone();
        let _id = id.clone();
        engine.register_fn("config", move || {
            let server = _server.clone();
            server.clone().get_plugin_config(_id.clone())
        });
        // println!("register type and fn cost: {:?}", t.elapsed());

        //? Initialize global variables
        engine.eval_ast_with_scope::<()>(&mut scope, &ast).unwrap();

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

        let res = self.engine.call_fn_with_options::<()>(
            CallFnOptions::new().eval_ast(false),
            &mut self.scope,
            &self.ast,
            &fn_name,
            args,
        );
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

    fn on_load(&mut self) {
        self.call_fn("on_load", ());
    }

    fn on_server_log(&mut self, content: String) {
        self.call_fn("on_server_log", (content,));
    }

    fn on_server_done(&mut self) {
        self.call_fn("on_server_done", ());
    }

    fn on_player_message(&mut self, player: String, msg: String) {
        self.call_fn("on_player_message", (player, msg));
    }

    fn on_call_fn(&mut self, fn_name: String) {
        self.call_fn(fn_name, ());
    }
}

#[derive(Clone, CustomType)]
#[rhai_type(extra = Self::build_extra)]
struct Server {
    inner: crate::server::Server,
}

impl Server {
    pub fn get_plugin_config(&mut self, id: String) -> rhai::Map {
        let config = self
            .inner
            .get_plugin_config(id)
            .cloned()
            .unwrap_or_default();
        let config = to_dynamic(config).unwrap();
        let config = from_dynamic(&config).unwrap();
        config
    }

    pub fn running(&mut self) -> bool {
        self.inner.running()
    }

    pub fn start(&mut self) -> Result<(), String> {
        self.inner.start()
    }

    pub fn stop(&mut self) -> Result<(), String> {
        self.inner.stop()
    }

    pub fn writeln(&mut self, content: String) {
        self.inner.writeln(&content)
    }

    pub fn say(&mut self, content: &str) {
        self.inner.say(content)
    }

    pub fn delay_call(&mut self, delay_ms: i64, plugin_id: String, fn_name: String) {
        self.inner.delay_call(delay_ms, plugin_id, fn_name)
    }

    pub fn tellraw<T: Into<Component>>(&mut self, target: String, component: T) {
        self.inner.tellraw(target, component)
    }

    pub fn add_log_filter(&mut self, filter: String) {
        self.inner.add_log_filter(Regex::new(&filter).unwrap())
    }

    fn build_extra(builder: &mut TypeBuilder<Self>) {
        builder
            .with_fn("start", Self::start)
            .with_fn("stop", Self::stop)
            .with_fn("delay_call", Self::delay_call)
            .with_fn("running", Self::running)
            .with_fn("say", Self::say)
            .with_fn("writeln", Self::writeln)
            .with_fn("tellraw", Self::tellraw::<String>)
            .with_fn("tellraw", Self::tellraw::<f64>)
            .with_fn("tellraw", Self::tellraw::<bool>)
            .with_fn("tellraw", Self::tellraw::<ComponentObject>)
            .with_fn("tellraw", Self::tellraw::<Vec<ComponentObject>>)
            .with_fn("add_log_filter", Self::add_log_filter);
    }
}
