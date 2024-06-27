#![feature(fs_try_exists)]

mod cli;
use clap::Parser;
use cli::Cli;

fn main() {
    env_logger::init();

    Cli::parse().exec();
}
