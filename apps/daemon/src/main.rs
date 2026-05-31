#[tokio::main]
async fn main() {
    daemon::cli::run().await;
}
