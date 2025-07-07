mod cli;
use async_compat::Compat;
use clap::Parser;
use cli::Cli;

fn main() {
    // env_logger::init();
    smol::block_on(Compat::new(Cli::parse().exec()));
}
