use clap::{Parser, Subcommand};

use crate::app;

#[derive(Parser)]
#[command(subcommand_required = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start,
}

pub async fn run() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Start => app::run().await,
    }
}
