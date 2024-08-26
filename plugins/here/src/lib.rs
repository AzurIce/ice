use ice_server::server::Server;

#[no_mangle]
pub fn hello() {
    println!("hello");
}

#[no_mangle]
pub fn hello_server(server: &mut Server) {
    println!("hello");
    println!("{}", server.running());
}

#[cfg(test)]
mod test {
    use ice_core::Loader;
    use ice_server::{config::Config, server::Server, Event};
    use std::{
        cell::RefCell,
        fs,
        sync::{Arc, Mutex, Once, OnceLock},
        thread,
    };
    use wasmtime::{Caller, Engine, Linker, Module, Store};
    use wasmtime_wasi::{preview1::WasiP1Ctx, WasiCtx, WasiCtxBuilder};

    struct MyState {
        num: u32,
        wasi: WasiCtx,
    }

    fn server() -> Arc<Mutex<Server>> {
        static SERVER: OnceLock<Arc<Mutex<Server>>> = OnceLock::new();
        SERVER
            .get_or_init(|| {
                let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
                let server = Server::new(
                    Config::new("server".to_string(), "1.21.1".to_string(), Loader::Quilt),
                    event_tx,
                );
                Arc::new(Mutex::new(server))
            })
            .clone()
    }

    fn server_running() -> bool {
        server().lock().unwrap().running()
    }

    #[test]
    fn test_hello() {
        // let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
        // let mut server = Server::new(
        //     Config::new("server".to_string(), "1.21.1".to_string(), Loader::Quilt),
        //     event_tx,
        // );

        let engine = Engine::default();

        // Modules can be compiled through either the text or binary format
        let module = fs::read("../../target/wasm32-wasip1/release/here.wasm").unwrap();
        // let module = r#"
        //     (module
        //         (import "host" "host_func" (func $host_hello (param i32)))

        //         (func (export "hello")
        //             i32.const 3
        //             call $host_hello)
        //     )
        // "#;
        let module = Module::new(&engine, module).unwrap();

        // Host functionality can be arbitrary Rust functions and is provided
        // to guests through a `Linker`.
        let mut linker = Linker::new(&engine);
        // wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |s| s).unwrap();
        linker
            .func_wrap("server", "running", || server_running())
            .unwrap();

        // let wasi = WasiCtxBuilder::new()
        //     .inherit_stdio()
        //     .inherit_args()
        //     .build_p1();

        // All wasm objects operate within the context of a "store". Each
        // `Store` has a type parameter to store host-specific data, which in
        // this case we're using `4` for.
        let mut store = Store::new(&engine, 4);

        // Instantiation of a module requires specifying its imports and then
        // afterwards we can fetch exports by name, as well as asserting the
        // type signature of the function with `get_typed_func`.
        let instance = linker.instantiate(&mut store, &module).unwrap();
        let hello = instance
            .get_typed_func::<(), ()>(&mut store, "hello")
            .unwrap();

        // And finally we can call the wasm!
        hello.call(&mut store, ()).unwrap();
    }
}
