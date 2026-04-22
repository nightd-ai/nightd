use axum::{Router, routing::get};
use tokio::net::TcpListener;

use crate::api::status;

pub(crate) async fn run() {
    let app = router();

    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to port 3000");

    println!("Server listening on http://127.0.0.1:3000");

    axum::serve(listener, app).await.expect("Server failed");
}

fn router() -> Router {
    Router::new().route("/status", get(status::get))
}
