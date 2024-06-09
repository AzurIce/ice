use std::{
    fs::{self, create_dir_all, DirEntry},
    path::{Path, PathBuf},
};

use crate::utils::{fs::copy_dir, time::get_cur_time_str};

use super::Server;

impl Server {
    pub fn del_snapshot(&self) {
        println!("[del_snapshop]");
        let snapshot_dir = Path::new("./backups").join("snapshots");
        if let Err(err) = create_dir_all(&snapshot_dir) {
            println!("failed to create all dir: {err}");
            return;
        }

        if let Ok(entries) = fs::read_dir(snapshot_dir) {
            let mut entries: Vec<DirEntry> =
                entries.into_iter().map(|entry| entry.unwrap()).collect();

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

    pub fn make_snapshot(&self) {
        let snapshot_dir = Path::new("./backups").join("snapshots");
        if let Err(err) = create_dir_all(snapshot_dir) {
            println!("failed to create all dir: {err}");
            return;
        }

        let src_path = Path::new(&self.config.dir).join("world");
        if !src_path.exists() {
            println!("skip world/, not exist");
            return;
        }

        let backup_name = get_cur_time_str();
        let dst_path = Path::new("./backups").join("snapshots").join(backup_name);
        println!("copying from {src_path:?} to {dst_path:?}...");
        if let Err(err) = copy_dir(&src_path, &dst_path) {
            println!("failed to copy: {err}")
        }
    }

    pub fn get_snapshot_list(&self) -> Vec<PathBuf> {
        let snapshot_dir = Path::new("./backups").join("snapshots");
        if let Err(err) = create_dir_all(&snapshot_dir) {
            println!("failed to create all dir: {err}");
            return Vec::new();
        }

        if let Ok(entries) = fs::read_dir(snapshot_dir) {
            let mut entries: Vec<DirEntry> =
                entries.into_iter().map(|entry| entry.unwrap()).collect();

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
}
