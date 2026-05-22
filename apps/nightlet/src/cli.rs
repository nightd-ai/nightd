use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "nightlet")]
#[command(about = "nightlet - the Nightlet agent")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Start the nightlet agent")]
    Start,
}

pub async fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start => {
            println!("nightlet started!");
        }
    }
}
