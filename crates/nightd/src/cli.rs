use crate::api::create_app;
use crate::db;
use crate::worker::Worker;
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::{error, info};

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
            start(host, port, concurrency, database).await;
        }
    }
}

async fn start(host: String, port: u16, concurrency: usize, database: PathBuf) {
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Failed to parse address");

    // Resolve database path - if relative, use data directory
    let database_path = if database.is_relative() {
        dirs::data_dir()
            .map(|d| d.join("nightd").join(&database))
            .unwrap_or_else(|| database)
    } else {
        database
    };

    // Ensure parent directory exists
    if let Some(parent) = database_path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent).expect("Failed to create database directory");
    }

    // Initialize database
    let database_url = format!("sqlite://{}", database_path.display());
    info!("Initializing database at {:?}", database_path);
    let pool = match db::init(&database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };
    info!("Database initialized successfully");

    // Start background worker
    info!(
        "Starting background worker with concurrency: {}",
        concurrency
    );
    let worker = match Worker::new(pool.clone(), concurrency).await {
        Ok(worker) => worker,
        Err(e) => {
            error!("Failed to initialize worker: {}", e);
            std::process::exit(1);
        }
    };

    // Spawn worker in background
    let worker_handle = tokio::spawn(async move {
        if let Err(e) = worker.run().await {
            error!("Worker error: {}", e);
        }
    });

    // Create API with database pool
    let app = create_app(pool);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!(
        "Starting nightd daemon on {} (concurrency: {}, database: {:?})",
        addr, concurrency, database_path
    );

    // Run server and worker concurrently
    tokio::select! {
        result = axum::serve(listener, app) => {
            if let Err(e) = result {
                error!("Server error: {}", e);
            }
        }
        _ = worker_handle => {
            error!("Worker terminated unexpectedly");
        }
    }
}
