pub mod bkarch;
pub mod bksnap;

use std::{fs, path::Path, thread, time::Duration};

use bksnap::make_snapshot;
use ice_util::fs::copy_dir;
use log::info;

use super::Core;

pub trait Command {
    fn cmd(&self) -> String;
    fn perform(&mut self, core: &mut Core, args: Vec<String>);
}

pub fn load_save<P: AsRef<Path>>(core: &mut Core, path: P) {
    let path = path.as_ref();
    info!("saving snapshot...");
    core.say("saving snapshot...");
    make_snapshot();
    info!("saved snapshot...");
    core.say("saved snapshot");

    info!("shutting down server to load snapshot...");
    core.say("shutting down server to load snapshot...");
    core.stop_server();

    while core.running_server.lock().unwrap().is_some() {
        thread::sleep(Duration::from_secs_f32(0.2));
    }
    info!("server stopped");

    info!("removing server/world");
    let world_path = Path::new("./server").join("world");
    fs::remove_dir_all(&world_path).unwrap();

    info!("copying snapshot from {:?}", path);
    copy_dir(path, &world_path).unwrap();
    info!("snapshot loaded, starting server");
    core.start_server();
}
