mod commands;
use clap::Parser;
use commands::Cli;
use tracing::{subscriber, Level};
use tracing_indicatif::IndicatifLayer;
use tracing_subscriber::{layer::{Filter, SubscriberExt}, util::SubscriberInitExt, FmtSubscriber};

fn main() {
    tracing_subscriber::fmt();
    
    let indicatif_layer = IndicatifLayer::new();


    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(indicatif_layer.get_stderr_writer()))
        .with(indicatif_layer)
        .init();

    Cli::parse().exec();
}
