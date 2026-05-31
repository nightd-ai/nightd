use axum::{Router, routing::get};
use tokio::{net::UnixListener, signal};

use crate::api;

pub(crate) async fn run() {
    let socket_path = std::env::var("NIGHTD_SOCKET_PATH").expect("NIGHTD_SOCKET_PATH must be set");

    if std::path::Path::new(&socket_path).exists() {
        std::fs::remove_file(&socket_path).expect("Failed to remove existing socket file");
    }

    let listener = UnixListener::bind(&socket_path).expect("Failed to bind to Unix socket");

    let app = Router::new().route("/info", get(api::info::info_handler));

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Server error");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
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
