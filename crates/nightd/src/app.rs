use crate::api;
use crate::db;
use crate::worker::Worker;
use std::path::PathBuf;
use tracing::{error, info};

pub(crate) async fn run(host: String, port: u16, concurrency: usize, database: PathBuf) {
    // Initialize database (now handles path resolution internally)
    info!("Initializing database at {:?}", database);
    let pool = match db::init(database).await {
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
    let router = api::router(pool);

    // Start server with graceful shutdown
    info!("Starting nightd daemon on {}:{}", host, port);

    // Run server and worker concurrently with graceful shutdown
    tokio::select! {
        result = api::run(router, &host, port) => {
            if let Err(e) = result {
                error!("Server error: {}", e);
            }
        }
        _ = worker_handle => {
            error!("Worker terminated unexpectedly");
        }
    }
}
