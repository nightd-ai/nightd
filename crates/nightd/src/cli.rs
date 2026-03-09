use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::app;

#[derive(Parser)]
#[command(name = "nightd")]
#[command(about = "Nightd daemon CLI")]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the nightd daemon
    Start {
        /// Host to bind to
        #[arg(short, long, default_value = "127.0.0.1")]
        host: String,
        /// Port to bind to
        #[arg(short, long, default_value = "8000")]
        port: u16,
        /// Number of concurrent tasks (default: 5)
        #[arg(short, long, default_value = "5")]
        concurrency: usize,
        /// SQLite database path (default: nightd.db)
        #[arg(short, long, default_value = "nightd.db")]
        database: PathBuf,
    },
}

pub async fn run() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start {
            host,
            port,
            concurrency,
            database,
        } => {
            app::run(host, port, concurrency, database).await;
        }
    }
}
