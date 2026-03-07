use crate::api::create_app;
use clap::{Parser, Subcommand};
use std::net::SocketAddr;

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
    },
}

pub async fn run() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { host, port } => {
            start(host, port).await;
        }
    }
}

async fn start(host: String, port: u16) {
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Failed to parse address");

    let app = create_app();

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Starting nightd daemon on {}", addr);

    axum::serve(listener, app).await.unwrap();
}
