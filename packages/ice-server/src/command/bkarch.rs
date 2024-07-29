use std::{
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};

use ice_util::{fs::copy_dir, time::get_cur_time_str};
use log::{error, info, warn};

use super::{load_save, Command};

#[derive(Default)]
pub struct BkArch;

impl Command for BkArch {
    fn cmd(&self) -> String {
        "bkarch".to_string()
    }

    fn perform(&mut self, core: &mut crate::Core, args: Vec<String>) {
        if args.is_empty() || args[0] == "list" {
            let archive_list = get_archive_list();

            core.say("archives: ");
            for (i, archive) in archive_list.into_iter().enumerate() {
                core.say(format!("{i}: {archive:?}"))
            }
        } else if args[0] == "make" {
            let comment = args[1..].join(" ");
            core.say("saving archive...");
            make_archive(&comment);
            core.say("saved archive")
        } else if args[0] == "load" {
            let idx = args.get(1).and_then(|s| s.parse::<usize>().ok());
            let snapshot_list = get_archive_list();
            if let Some(target_snapshot) = idx.and_then(|idx| snapshot_list.get(idx)) {
                load_save(core, target_snapshot)
            } else {
                core.say("please provide a valid snapshot index, use `bksnap list` to show snapshot list");
            }
        }
    }
}

pub fn get_archive_list() -> Vec<PathBuf> {
    let dir = Path::new("./backups").join("archives");
    if let Err(err) = fs::create_dir_all(&dir) {
        error!("failed to create all dir: {err}");
        return Vec::new();
    }

    if let Ok(entries) = fs::read_dir(dir) {
        let mut entries: Vec<DirEntry> = entries.into_iter().map(|entry| entry.unwrap()).collect();

        entries.sort_by_key(|entry| entry.metadata().unwrap().created().unwrap());
        entries
            .into_iter()
            .map(|entry| entry.path())
            .collect::<Vec<PathBuf>>()
    } else {
        Vec::new()
    }
}

pub fn make_archive(name: &str) {
    let dir = Path::new("./backups").join("archives");
    if let Err(err) = fs::create_dir_all(&dir) {
        error!("failed to create all dir: {err}");
        return;
    }

    let src_path = Path::new(&"./server/").join("world");
    if !src_path.exists() {
        warn!("skip world/, not exist");
        return;
    }

    let backup_name = format!("{} {}", get_cur_time_str(), name);
    let dst_path = dir.join(backup_name);
    info!("copying from {src_path:?} to {dst_path:?}...");
    if let Err(err) = copy_dir(&src_path, &dst_path) {
        error!("failed to copy: {err}")
    }
}
