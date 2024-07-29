use std::{
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};

use ice_util::{fs::copy_dir, time::get_cur_time_str};
use log::{error, info, warn};

use crate::Core;

use super::Command;

pub struct BkSnap {
    max_slots: usize,
}

impl Default for BkSnap {
    fn default() -> Self {
        Self { max_slots: 6 }
    }
}

impl BkSnap {
    pub fn max_slots(mut self, max_slots: usize) -> Self {
        self.max_slots = max_slots;
        self
    }
}

impl Command for BkSnap {
    fn cmd(&self) -> String {
        "bksnap".to_string()
    }

    fn perform(&mut self, core: &mut Core, args: Vec<String>) {
        if args.is_empty() || args[0] == "list" {
            let snapshot_list = get_snapshot_list();

            core.say("snapshots: ");
            for (i, snapshot) in snapshot_list.into_iter().enumerate() {
                core.say(format!("{i}: {snapshot:?}"))
            }
        } else if args[0] == "make" {
            while get_snapshot_list().len() >= self.max_slots {
                del_snapshot()
            }
            core.say("saving snapshot...");
            make_snapshot();
            core.say("saved snapshot")
        } else if args.len() == 2 && args[0] == "load" {
            println!("bksnap load, not implemented yet")
            // TODO: load snap backup
        }
    }
}

// Backup related
pub fn del_snapshot() {
    info!("deleting snapshot");
    let snapshot_dir = Path::new("./backups").join("snapshots");
    if let Err(err) = fs::create_dir_all(&snapshot_dir) {
        println!("failed to create all dir: {err}");
        return;
    }

    if let Ok(entries) = fs::read_dir(snapshot_dir) {
        let mut entries: Vec<DirEntry> = entries.into_iter().map(|entry| entry.unwrap()).collect();

        entries.sort_by_key(|entry| entry.metadata().unwrap().created().unwrap());
        let entries = entries
            .into_iter()
            .map(|entry| entry.path())
            .collect::<Vec<PathBuf>>();
        if let Some(first) = entries.first() {
            println!("[del_snapshop]: Deleting {first:?}...");
            if let Err(err) = fs::remove_dir_all(first) {
                println!("Failed to remove dir: {err}")
            }
            println!("[del_snapshop]: Snapshop deleted");
        }
    }
}

pub fn make_snapshot() {
    let snapshot_dir = Path::new("./backups").join("snapshots");
    if let Err(err) = fs::create_dir_all(snapshot_dir) {
        error!("failed to create all dir: {err}");
        return;
    }

    let src_path = Path::new(&"./server/").join("world");
    if !src_path.exists() {
        warn!("skip world/, not exist");
        return;
    }

    let backup_name = get_cur_time_str();
    let dst_path = Path::new("./backups").join("snapshots").join(backup_name);
    info!("copying from {src_path:?} to {dst_path:?}...");
    if let Err(err) = copy_dir(&src_path, &dst_path) {
        error!("failed to copy: {err}")
    }
}

pub fn get_snapshot_list() -> Vec<PathBuf> {
    let snapshot_dir = Path::new("./backups").join("snapshots");
    if let Err(err) = fs::create_dir_all(&snapshot_dir) {
        error!("failed to create all dir: {err}");
        return Vec::new();
    }

    if let Ok(entries) = fs::read_dir(snapshot_dir) {
        let mut entries: Vec<DirEntry> = entries.into_iter().map(|entry| entry.unwrap()).collect();

        entries.sort_by_key(|entry| entry.metadata().unwrap().created().unwrap());
        entries
            .into_iter()
            .map(|entry| entry.path())
            .collect::<Vec<PathBuf>>()
    } else {
        Vec::new()
    }
    // snapshot_list.sort_by_key(|snapshot|snapshot.metadata.created().unwrap());
    // snapshot_list
}
