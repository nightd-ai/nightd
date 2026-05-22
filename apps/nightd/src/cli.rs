use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nightd")]
#[command(about = "nightd - the Nightd daemon")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start the nightd daemon")]
    Start,
}

pub async fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start => {
            println!("nightd started!");
        }
    }
}
