use clap::{Parser, Subcommand};
use nightd_client::Client;

#[derive(Parser)]
#[command(subcommand_required = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Info,
}

pub async fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info => {
            let client = Client::new().expect("failed to create daemon client");
            let info = client.info().await.expect("failed to fetch info");
            println!("Status: {}", info.status);
        }
    }
}
