use clap::{Parser, Subcommand};

use crate::server;

#[derive(Parser)]
#[command(name = "nightd")]
#[command(about = "A daemon to schedule autonomous coding agents")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the daemon
    Start,
}

pub async fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start => {
            server::run().await;
        }
    }
}
