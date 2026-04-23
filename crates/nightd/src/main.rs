use std::net::{Ipv4Addr, SocketAddr};
use std::str::FromStr;

use clap::{Parser, Subcommand};
use nightd::api;
use sqlx::SqlitePool;
use sqlx::sqlite::SqliteConnectOptions;
use tokio::net::TcpListener;
use tokio::signal;

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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Start => {
            let db_options = SqliteConnectOptions::from_str("sqlite:nightd.db")
                .expect("Failed to parse database URL")
                .create_if_missing(true);
            let db = SqlitePool::connect_with(db_options)
                .await
                .expect("Failed to connect to database");
            let (session_tx, executor_token) = agent::run_executor(&db).await;
            let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 3000u16));
            let listener = TcpListener::bind(addr)
                .await
                .expect("Failed to bind to port 3000");
            let app = api::router(db, session_tx);
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal())
                .await
                .expect("Server failed");
            // after server gracefully shuts down, cancel the executor
            executor_token.cancel();
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
