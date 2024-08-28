use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use rune::alloc::clone::TryClone;
use rune::alloc::HashMap;
use rune::runtime::ConstValue;
use rune::runtime::GuardedArgs;
use rune::runtime::Object;
use rune::runtime::RuntimeContext;
use rune::termcolor::ColorChoice;
use rune::termcolor::StandardStream;
use rune::Any;
use rune::Context;
use rune::ContextError;
use rune::Diagnostics;
use rune::FromValue;
use rune::Module;
use rune::Source;
use rune::Sources;
use rune::ToValue;
use rune::Unit;
use rune::Value;
use rune::Vm;
use tracing::error;
use tracing::info;

use super::Plugin;

#[derive(Any)]
struct Server {
    inner: crate::server::Server,
}

impl Server {
    #[rune::function]
    pub fn running(&self) -> bool {
        self.inner.running()
    }

    #[rune::function]
    pub fn start(&self) -> Result<(), String> {
        self.inner.start()
    }

    #[rune::function]
    pub fn stop(&mut self) -> Result<(), String> {
        self.inner.stop()
    }

    #[rune::function]
    pub fn writeln(&mut self, content: String) {
        self.inner.writeln(&content)
    }

    #[rune::function]
    pub fn say(&mut self, content: String) {
        self.inner.say(&content)
    }
}

pub fn server_api() -> Result<Module, ContextError> {
    let mut module = Module::new();
    module.ty::<Server>()?;
    module.function_meta(Server::running)?;
    module.function_meta(Server::start)?;
    module.function_meta(Server::stop)?;
    module.function_meta(Server::writeln)?;
    module.function_meta(Server::say)?;
    Ok(module)
}
pub struct RunePlugin {
    id: String,
    server: Server,
    runtime: Arc<RuntimeContext>,
    unit: Arc<Unit>,
    state: ConstValue,
}

impl RunePlugin {
    /*
        engine initializing cost: 8.72425ms
        ast compile cost: 1.621958ms
        first eval and id cost: 457.25µs
        register type and fn cost: 236.625µs
    */
    pub fn from_file(server: crate::server::Server, path: PathBuf) -> Self {
        // let context = rune_context();
        let mut context = Context::with_default_modules().unwrap();
        let mut module = server_api().unwrap();

        // println!("state: {:?}", state);
        let state = Arc::new(Mutex::new(Vec::new()));
        let _state = state.clone();
        module
            .function("get_state", move || {
                let data = bitcode::deserialize::<Value>(&_state.lock().unwrap())
                    .unwrap_or(ConstValue::Object(HashMap::new()).into_value().unwrap());
                println!("get_state: {:?}", data);
                data
            })
            .build()
            .unwrap();
        let _state = state.clone();
        module
            .function("set_state", move |value: Value| {
                let data = bitcode::serialize(&value).unwrap();
                println!("set_state: {:?}", value);
                *_state.lock().unwrap() = data;
            })
            .build()
            .unwrap();
        context.install(module).unwrap();

        let t = Instant::now();
        let runtime = Arc::new(context.runtime().unwrap());
        println!("runtime cost: {:?}", t.elapsed());

        let mut sources = Sources::new();
        sources.insert(Source::from_path(&path).unwrap()).unwrap();

        let t = Instant::now();
        let mut diagnostics = Diagnostics::new();
        let res = rune::prepare(&mut sources)
            .with_context(&context)
            .with_diagnostics(&mut diagnostics)
            .build();
        if !diagnostics.is_empty() {
            let mut writer = StandardStream::stderr(ColorChoice::Always);
            diagnostics.emit(&mut writer, &sources).unwrap();
        }
        let unit = Arc::new(res.unwrap());
        println!("compile unit cost: {:?}", t.elapsed());

        let t = Instant::now();
        let mut vm = Vm::new(runtime.clone(), unit.clone());
        let id = vm.call(["id"], ()).unwrap();
        let id = id
            .into_string()
            .into_result()
            .unwrap()
            .take()
            .unwrap()
            .into_std();
        println!("call id cost: {:?}", t.elapsed());

        let state = vm.call(["init"], ()).unwrap();
        let state = ConstValue::from_value(state).into_result().unwrap();
        // let state = Arc::new(Mutex::new(state));

        let server = Server { inner: server };

        Self {
            id,
            server,
            state,
            runtime,
            unit,
        }
    }

    fn vm(&self) -> Vm {
        Vm::new(self.runtime.clone(), self.unit.clone())
    }

    /// Calls a function in the plugin, skip if not exist
    pub fn call_fn<T: FromValue + Debug>(
        &mut self,
        fn_name: impl AsRef<str>,
        args: impl GuardedArgs,
    ) {
        let vm = Vm::new(self.runtime.clone(), self.unit.clone());
        let fn_name = fn_name.as_ref();
        call_rune_fn_if_exist(vm, fn_name, args)
    }
}

fn call_rune_fn_if_exist(mut vm: Vm, fn_name: &str, args: impl GuardedArgs) {
    let res = vm.call([fn_name], args);
    if let Ok(_) = vm.lookup_function([fn_name]) {
        if let Err(err) = res {
            error!("{:?}", err);
        } else {
            info!("{:?}", res);
        }
    }
}

impl Plugin for RunePlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn on_load(&mut self) {
        let vm = self.vm();
        call_rune_fn_if_exist(vm, "on_load", (&mut self.server,));
    }

    fn on_server_log(&mut self, content: String) {
        let vm = self.vm();
        call_rune_fn_if_exist(vm, "on_server_log", (&mut self.server, content));
    }

    fn on_server_done(&mut self) {
        let vm = self.vm();
        call_rune_fn_if_exist(vm, "on_load", (&mut self.server,));
    }

    fn on_player_message(&mut self, player: String, msg: String) {
        let vm = self.vm();
        call_rune_fn_if_exist(vm, "on_player_message", (&mut self.server, player, msg));
    }
}
