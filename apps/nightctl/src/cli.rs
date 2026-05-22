use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nightctl")]
#[command(about = "nightctl - CLI for Nightd")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Show info about nightd")]
    Info,
}

pub async fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info => {
            println!("nightd info: daemon is ready");
        }
    }
}
